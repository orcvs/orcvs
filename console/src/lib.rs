#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod source;
mod style;
pub use app::ConsoleApp;
use app::Coord;

// pub fn distance(Coord(x1, y1): Coord, Coord(x2, y2): Coord) -> f64 {
//     let x = (x2 as f64 - x1 as f64) as f64;
//     let y = (y2 as f64 - y1 as f64) as f64;

//     let d = y.hypot(x);
//     d.floor()
// }

#[cfg(test)]
mod test {
    use std::sync::Once;

    // use crate::{app::Coord, distance};

    #[test]
    fn test_distance() {
        trace();

        // let a = Coord(1, 1);
        // let b = Coord(1, 2);
        // let d = distance(a, b);

        // assert_eq!(d, 1.0);

        // let a = Coord(1, 1);
        // let b = Coord(3, 3);
        // let d = distance(a, b);

        // assert_eq!(d, 2.0);

        // let a = Coord(1, 1);
        // let b = Coord(7, 5);
        // let d = distance(a, b);

        // assert_eq!(d, 7.0);
    }

    #[allow(dead_code)]
    static INIT: Once = Once::new();

    #[allow(dead_code)]
    pub fn trace() {
        INIT.call_once(|| {
            use tracing_subscriber::FmtSubscriber;

            let subscriber = FmtSubscriber::builder()
                .with_max_level(tracing::Level::DEBUG) // Set the maximum level of tracing events that should be logged.
                .with_line_number(true)
                .with_target(true)
                .finish();

            tracing::subscriber::set_global_default(subscriber)
                .expect("setting default subscriber failed");
        });
    }
}
