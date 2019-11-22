use yew::ShouldRender;

/// Alternative to using Message enums.
///
/// Using Effects instead of Messages allows you to define the mutation to your component's state
/// from inside `html!` macros instead of from within update functions.
pub struct Effect<COMP>(Box<dyn Fn(&mut COMP) -> ShouldRender>);

impl <COMP> Default for Effect<COMP> {
    fn default() -> Self {
        Effect::new(|_| false)
    }
}

impl <COMP> Effect<COMP> {
    /// Wraps a function in an Effect wrapper.
    pub fn new(f: impl Fn(&mut COMP)-> ShouldRender + 'static) -> Self {
        Effect(Box::new(f))
    }

    /// Runs the effect, causing a mutation to the component state.
    pub fn call(self, component: &mut COMP) -> ShouldRender {
        (self.0)(component)
    }
}

/// Terser wrapper function to be used instead of `Effect::new()`.
pub fn effect<COMP>(f: impl Fn(&mut COMP) -> ShouldRender + 'static ) -> Effect<COMP>
{
    Effect::new(f)
}