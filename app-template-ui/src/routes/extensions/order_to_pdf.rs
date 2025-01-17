use leptos::*;
use saleor_app_sdk::bridge::{action::PayloadRedirect, dispatch_event, AppBridge};

#[component]
pub fn OrderToPdf(bridge: ReadSignal<Option<AppBridge>>) -> impl IntoView {
    view! {
        <h1>Yello!</h1>

        {move || match bridge() {
            Some(bridge) => {
                match bridge.state.ready {
                    true => view!{
                        <div>
                        <button on:click=move |_|{
                            dispatch_event(saleor_app_sdk::bridge::action::Action::Redirect(PayloadRedirect{
                                to: format!("/apps/{}/app", bridge.state.id),
                                new_context: None
                            })).expect("failed sending redirect action");
                        }>Settings</button>
                            <p class="italic text-lg">"token:"{bridge.state.token}</p>
                        </div>
                    }.into_view(),
                    false => view!{<p class="italic text-lg">r#"(bridge exists) Loading AppBridge, please wait..."#</p>}.into_view()
                }
            },
                None => view!{<p class="italic text-lg">r#"Loading AppBridge, please wait..."#</p>}.into_view()
            }}
    }
}
