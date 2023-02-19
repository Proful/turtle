use turtle::Turtle;

fn main() {
    let mut turtle = Turtle::new();

    for _ in 0..360 {
        turtle.forward(0.2);
        turtle.right(1.0);
    }

    turtle.hide();
    turtle.pen_up();
    turtle.forward(40.0);
    turtle.right(90.0);
    turtle.forward(10.0);
    turtle.left(90.0);
    turtle.pen_down();
    turtle.show();
    turtle.set_speed("slower");
    turtle.forward(50.0);
}
