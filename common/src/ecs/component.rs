pub trait Component {
    fn id() -> u32 where Self: Sized;
}