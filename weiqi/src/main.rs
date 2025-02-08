use bevy::{
    color::palettes::css::WHITE, ecs::schedule::ExecutorKind, prelude::*, window::WindowTheme,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "weiqi".into(),
                window_theme: Some(WindowTheme::Light),
                ..default()
            }),
            ..default()
        }))
        // .add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_secs(5)))
        // 仅对更新计划禁用多线程
        .edit_schedule(Update, |schedule| {
            schedule.set_executor_kind(ExecutorKind::SingleThreaded);
        })
        .add_systems(Startup, setup)
        .add_systems(Startup, (setup_board, setup_player))
        .add_systems(Update, draw_cursor)
        .add_systems(Update, (draw_board, handle_click))
        .run();
}

/// 绘制光标
fn draw_cursor(
    camera_query: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
    mut gizmos: Gizmos,
) {
    let (camera, camera_transform) = *camera_query;

    // 获取当前窗口中的光标位置
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let Ok(point) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    // 白色圆形光标
    gizmos.circle_2d(point, 10., WHITE);
}

/// 退出应用
fn exit(mut exit: EventWriter<AppExit>) {
    exit.send(AppExit::Success);
}

/// 设置相机
fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// 绘制棋盘
fn draw_board(mut gizmos: Gizmos) {
    // 棋盘大小
    let board_size = 19;
    // 网格大小
    let grid_size = 30.0;
    let offset = -((board_size as f32 * grid_size) / 2.0);

    for i in 0..board_size {
        let x = offset + i as f32 * grid_size;
        gizmos.line_2d(
            Vec2::new(x, offset),
            Vec2::new(x, offset + (board_size - 1) as f32 * grid_size),
            Color::BLACK,
        );
        let y = offset + i as f32 * grid_size;
        gizmos.line_2d(
            Vec2::new(offset, y),
            Vec2::new(offset + (board_size - 1) as f32 * grid_size, y),
            Color::BLACK,
        );
    }
}

/// 棋子组件
#[derive(Component)]
struct Piece {
    color: PieceColor,
}

/// 棋子颜色枚举
#[derive(PartialEq, Clone, Copy)]
enum PieceColor {
    Black,
    White,
}

/// 生成棋子
fn spawn_piece(
    commands: &mut Commands,
    x: f32,
    y: f32,
    color: PieceColor,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    // commands.spawn((
    //     Sprite {
    //         color: match color {
    //             PieceColor::Black => Color::BLACK,
    //             PieceColor::White => Color::WHITE,
    //         },
    //         custom_size: Some(Vec2::new(20.0, 20.0)),
    //         ..default()
    //     },
    //     Transform::from_xyz(x, y, 1.0),
    //     GlobalTransform::default(),
    //     Piece { color },
    // ));

    commands.spawn((
        Mesh2d(meshes.add(Circle::new(12.0))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(match color {
            PieceColor::Black => Color::BLACK,
            PieceColor::White => Color::WHITE,
        }))),
        Transform::from_xyz(x, y, 1.0),
        // GlobalTransform::default(),
        Piece { color },
    ));
}

/// 棋盘状态
#[derive(Resource)]
struct BoardState {
    grid: [[Option<PieceColor>; 19]; 19],
}

/// 初始化棋盘状态
fn setup_board(mut commands: Commands) {
    commands.insert_resource(BoardState {
        grid: [[None; 19]; 19],
    });
}

/// 当前玩家
#[derive(Resource)]
struct CurrentPlayer(PieceColor);

/// 初始化玩家
fn setup_player(mut commands: Commands) {
    commands.insert_resource(CurrentPlayer(PieceColor::Black));
}

/// 处理点击事件
fn handle_click(
    mut commands: Commands,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut board_state: ResMut<BoardState>,
    mut current_player: ResMut<CurrentPlayer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // 处理鼠标左键按下
    if mouse_button_input.just_pressed(MouseButton::Left) {
        let window = windows.single();
        let (camera, camera_transform) = camera_query.single();

        let Some(world_position) = window.cursor_position().map(|cursor| {
            camera
                .viewport_to_world_2d(camera_transform, cursor)
                .unwrap()
        }) else {
            return;
        };

        let grid_size = 30.0;
        let board_size = 19;
        let offset = -((board_size as f32 * grid_size) / 2.0);

        let i = ((world_position.x - offset) / grid_size).round() as usize;
        let j = ((world_position.y - offset) / grid_size).round() as usize;

        if i < board_size && j < board_size && board_state.grid[i][j].is_none() {
            let x = offset + i as f32 * grid_size;
            let y = offset + j as f32 * grid_size;
            spawn_piece(
                &mut commands,
                x,
                y,
                current_player.0,
                &mut meshes,
                &mut materials,
            );
            board_state.grid[i][j] = Some(current_player.0);

            // 切换玩家
            current_player.0 = match current_player.0 {
                PieceColor::Black => PieceColor::White,
                PieceColor::White => PieceColor::Black,
            };
        }
    }
}
