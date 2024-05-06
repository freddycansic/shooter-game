use winit::event_loop::EventLoop;

pub trait Application {
    fn run(self, event_loop: EventLoop<()>);
    fn update(&mut self);
    fn render(&mut self);
    fn render_gui(&mut self);
}
