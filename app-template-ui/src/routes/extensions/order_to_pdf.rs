use leptos::*;
use leptos_dom::logging::{console_error, console_log};

#[component]
pub fn Pdf() -> impl IntoView {
    let bridge = create_effect(|_| {
        use saleor_app_sdk::bridge::AppBridge;
        use saleor_app_sdk::{
            bridge::action::{Action, PayloadRequestPermissions},
            manifest::AppPermission,
        };
        match AppBridge::new(true) {
            Ok(mut app_bridge) => {
                console_log("App Bridge connected");
                let cb_handle = app_bridge
                    .listen_to_events(|event| match event {
                        Ok(event) => console_log(&format!("order_to_pdf: {:?}", event)),
                        Err(e) => console_error(&format!("order_to_pdf: {:?}", e)),
                    })
                    .unwrap();
                //TODO: imagine leaking memory on purpose xd
                cb_handle.forget();
                _ = app_bridge.dispatch_event(Action::RequestPermissions(
                    PayloadRequestPermissions {
                        permissions: vec![AppPermission::ManageOrders],
                        redirect_path: "".to_owned(),
                    },
                ));
            }
            Err(e) => console_error(&format!("{:?}", e)),
        }; // let mut bridge = bridge.unwrap();

        // match bridge.dispatch_event(Event::Handshake(PayloadHanshake::default())) {
        //     Ok(ev) => {
        //         console_log(&format! {"{:?}",ev});
        //     }
        //     Err(e) => {
        //         console_log(&format! {"{:?}",e});
        //     }
        // };
        // async fn temp(){}
        // temp()
    });
    view! {
        <h1>Yello!</h1>
        <p class="italic text-lg">r#"Loading AppBridge, please wait..."#</p>
    }
}
