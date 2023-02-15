use yew::{Component, html};

pub struct App;
impl Component for App {
    type Message = ();
    type Properties = ();

    fn create(ctx: &yew::Context<Self>) -> Self { Self }

    fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        false
    }

    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
        html! {
            <h1>{ "Hello, world!" }</h1>
        }
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
