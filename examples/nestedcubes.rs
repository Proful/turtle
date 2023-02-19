use turtle::{Color, Drawing};

fn main() {
    let mut drawing = Drawing::new();
    let mut turtle = drawing.add_turtle();

    drawing.set_background_color("black");
    turtle.set_speed(20);
    turtle.set_pen_size(4.0);
    for i in 0..290 {
        turtle.set_pen_color(Color {
            red: (i as f64 / 300.0 * 4.0) * 255.0 % 255.0,
            green: 255.0,
            blue: 255.0,
            alpha: 0.5,
        });
        turtle.forward(i as f64);

        turtle.right(60.0);
    }
}
