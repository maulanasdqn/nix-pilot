use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

use crate::components::layout::Layout;
use crate::pages::{
    AddFlakePage, AddMachinePage, AddSecretPage, DashboardPage, DeployWizardPage, FlakeDetailPage,
    FlakeListPage, InstallWizardPage, KeysPage, MachineDetailPage, MachineListPage,
    NixOperationsPage, NotFoundPage, SecretsListPage, ServiceDetailPage, ServiceListPage,
    ServiceLogsPage, SettingsPage,
};

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
            <Layout>
                <Routes fallback=|| view! { <NotFoundPage/> }>
                    <Route path=path!("/") view=DashboardPage/>
                    <Route path=path!("/machines") view=MachineListPage/>
                    <Route path=path!("/machines/add") view=AddMachinePage/>
                    <Route path=path!("/machines/:id") view=MachineDetailPage/>
                    <Route path=path!("/flakes") view=FlakeListPage/>
                    <Route path=path!("/flakes/add") view=AddFlakePage/>
                    <Route path=path!("/flakes/:id") view=FlakeDetailPage/>
                    <Route path=path!("/install") view=InstallWizardPage/>
                    <Route path=path!("/deploy") view=DeployWizardPage/>
                    // Services
                    <Route path=path!("/services/:machine_id") view=ServiceListPage/>
                    <Route path=path!("/services/:machine_id/:service") view=ServiceDetailPage/>
                    <Route path=path!("/services/:machine_id/:service/logs") view=ServiceLogsPage/>
                    // Nix Operations
                    <Route path=path!("/nix") view=NixOperationsPage/>
                    // Secrets Management
                    <Route path=path!("/secrets") view=SecretsListPage/>
                    <Route path=path!("/secrets/add") view=AddSecretPage/>
                    <Route path=path!("/secrets/keys") view=KeysPage/>
                    // Settings
                    <Route path=path!("/settings") view=SettingsPage/>
                </Routes>
            </Layout>
        </Router>
    }
}
