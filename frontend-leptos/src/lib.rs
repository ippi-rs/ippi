use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Router>
            <Routes>
                <Route path="/" view=Home/>
                <Route path="/stream" view=Stream/>
                <Route path="/settings" view=Settings/>
            </Routes>
        </Router>
    }
}

#[component]
fn Home() -> impl IntoView {
    view! {
        <Title text="IPPI - KVM over IP"/>
        <div class="container">
            <h1>"IPPI"</h1>
            <p>"KVM over IP for Raspberry Pi"</p>
            <nav>
                <A href="/stream">"Stream"</A>
                <A href="/settings">"Settings"</A>
            </nav>
        </div>
    }
}

#[component]
fn Stream() -> impl IntoView {
    view! {
        <Title text="Stream - IPPI"/>
        <div class="stream">
            <h2>"Video Stream"</h2>
            <div class="video-placeholder">"No stream connected"</div>
        </div>
    }
}

#[component]
fn Settings() -> impl IntoView {
    view! {
        <Title text="Settings - IPPI"/>
        <div class="settings">
            <h2>"Settings"</h2>
            <p>"Configuration panel"</p>
        </div>
    }
}

pub mod entry {
    use leptos::view;
    use leptos_meta::*;
    use leptos_router::*;

    #[component]
    pub fn App() -> impl IntoView {
        provide_meta_context();
        view! { <Router> <Routes> <Route path="/" view=super::Home/> </Routes> </Router> }
    }
}
