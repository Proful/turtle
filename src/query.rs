use color::Color;
use state::Path;

pub enum Query {
    Request(Request),
    Drawing(DrawingCommand),
}

pub enum Request {
    TurtleState,
    DrawingState,
}

pub enum DrawingCommand {
    /// When a path is finished being animated, it needs to be persisted in the renderer
    /// so it can be redrawn every frame
    StorePath(Path),
    /// Begins filling with the current fill color from the next path onwards. If temporary_path is
    /// set, it is included in the fill shape. Any paths sent via StorePath will be added to the
    /// filled shape.
    /// This command should be passed the fill color that was set at the time when this command
    /// was issued. We cannot simply poll that from the state when this command is handled because
    /// that may be well after the fill color has changed.
    BeginFill(Color),
    /// Send EndFill to finish the filled shape.
    EndFill,
    /// Clears the image completely
    ///
    /// Panics if temporary_path is not None
    Clear,
}
