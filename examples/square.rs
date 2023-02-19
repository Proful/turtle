use turtle::Drawing;

fn main() {
    let mut drawing = Drawing::new();
    let mut turtle = drawing.add_turtle();
    drawing.set_background_color("black");
    turtle.set_pen_color("white");
    turtle.set_pen_size(1.0);

    for _ in 1..=4 {
        turtle.forward(200.0);
        turtle.right(90.0);
        turtle.wait(2.0);
        turtle.wait_for_click();
    }
}
