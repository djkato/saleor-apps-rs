use leptos::*;

#[component]
pub fn Home() -> impl IntoView {
    view! {
        <h1>Yello!</h1>
        <p class="italic text-lg">
            r#"This is the home page for app-template-ui. You're not bridged with the Dashboard, so I can't show you anything, sorry!"#
        </p>
    }
}
