use leptos::prelude::*;
use saleor_app_sdk::bridge::AppBridge;

#[component]
pub fn OrderToPdf(bridge: ReadSignal<Option<AppBridge>>) -> impl IntoView {
    view! {
        <h1>Yello!</h1>

        {move || match bridge.get() {
            Some(bridge) => {
                match bridge.state.ready {
                    true => {
                        view! {
                            <div>
                                <p class="italic text-lg">"token:"{bridge.state.token}</p>
                            </div>
                        }
                            .into_any()
                    }
                    false => {
                        view! {
                            <p class="italic text-lg">
                                r#"(bridge exists) Loading AppBridge, please wait..."#
                            </p>
                        }
                            .into_any()
                    }
                }
            }
            None => {
                view! { <p class="italic text-lg">r#"Loading AppBridge, please wait..."#</p> }
                    .into_any()
            }
        }}
    }
}
