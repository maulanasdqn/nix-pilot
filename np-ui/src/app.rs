use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

use crate::components::layout::Layout;
use crate::pages::{
    AddFlakePage, AddSecretPage, DashboardPage, FlakeDetailPage, FlakeListPage, KeysPage,
    LoginPage, NixOperationsPage, NotFoundPage, RebuildPage, SecretsListPage, ServiceDetailPage,
    ServiceListPage, ServiceLogsPage, SettingsPage,
};

/// Check if user is authenticated by checking localStorage for token
pub fn is_authenticated() -> bool {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            if let Ok(Some(_token)) = storage.get_item("np_token") {
                return true;
            }
        }
    }
    false
}

/// Redirect to login if not authenticated
pub fn redirect_to_login() {
    if let Some(window) = web_sys::window() {
        let _ = window.location().set_href("/login");
    }
}

/// Logout - clear token and redirect to login
pub fn logout() {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.remove_item("np_token");
        }
        let _ = window.location().set_href("/login");
    }
}

/// Protected page wrapper component
#[component]
pub fn ProtectedPage(children: Children) -> impl IntoView {
    let authenticated = is_authenticated();

    if !authenticated {
        redirect_to_login();
        // Return a loading spinner while redirecting
        return view! {
            <div class="min-h-screen bg-background flex items-center justify-center">
                <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
            </div>
        }.into_any();
    }

    // User is authenticated, render the layout with children
    view! {
        <Layout>
            {children()}
        </Layout>
    }.into_any()
}

/// Root application component
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/np-ui.css"/>
        <Title text="Nix Pilot - NixOS Management"/>
        <Meta name="description" content="Web UI for managing NixOS deployments, flakes, and services"/>
        <Meta name="viewport" content="width=device-width, initial-scale=1"/>

        <Router>
            <Routes fallback=|| view! { <NotFoundPage/> }>
                // Public route - Login page (no layout)
                <Route path=path!("/login") view=LoginPage/>

                // Root redirects to dashboard
                <Route path=path!("/") view=|| {
                    if let Some(window) = web_sys::window() {
                        let _ = window.location().set_href("/dashboard");
                    }
                    view! { <div class="min-h-screen bg-background"></div> }
                }/>

                // Protected routes
                <Route path=path!("/dashboard") view=|| view! { <ProtectedPage><DashboardPage/></ProtectedPage> }/>
                <Route path=path!("/services") view=|| view! { <ProtectedPage><ServiceListPage/></ProtectedPage> }/>
                <Route path=path!("/services/:service") view=|| view! { <ProtectedPage><ServiceDetailPage/></ProtectedPage> }/>
                <Route path=path!("/services/:service/logs") view=|| view! { <ProtectedPage><ServiceLogsPage/></ProtectedPage> }/>
                <Route path=path!("/flakes") view=|| view! { <ProtectedPage><FlakeListPage/></ProtectedPage> }/>
                <Route path=path!("/flakes/add") view=|| view! { <ProtectedPage><AddFlakePage/></ProtectedPage> }/>
                <Route path=path!("/flakes/:id") view=|| view! { <ProtectedPage><FlakeDetailPage/></ProtectedPage> }/>
                <Route path=path!("/rebuild") view=|| view! { <ProtectedPage><RebuildPage/></ProtectedPage> }/>
                <Route path=path!("/nix") view=|| view! { <ProtectedPage><NixOperationsPage/></ProtectedPage> }/>
                <Route path=path!("/secrets") view=|| view! { <ProtectedPage><SecretsListPage/></ProtectedPage> }/>
                <Route path=path!("/secrets/add") view=|| view! { <ProtectedPage><AddSecretPage/></ProtectedPage> }/>
                <Route path=path!("/secrets/keys") view=|| view! { <ProtectedPage><KeysPage/></ProtectedPage> }/>
                <Route path=path!("/settings") view=|| view! { <ProtectedPage><SettingsPage/></ProtectedPage> }/>
            </Routes>
        </Router>
    }
}
