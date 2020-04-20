pub mod actions;
pub mod ecs;
pub mod math;
pub mod net;

pub static PLAYER_COLORS: [[f32; 3]; 5] = [
    [0.64, 0.12, 0.11],
    [0.04, 0.45, 0.69],
    [0.0, 0.49, 0.26],
    [0.40, 0.3, 0.55],
    [0.57, 0.57, 0.57],
];

#[macro_export]
macro_rules! profile_scope {
    ($string:expr) => {
        #[cfg(feature = "profiler")]
        let _profile_scope =
            thread_profiler::ProfileScope::new(format!("{}: {}", module_path!(), $string));
    };
}
