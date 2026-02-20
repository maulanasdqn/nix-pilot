use leptos::prelude::*;
use leptos_router::components::A;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;

use crate::components::common::Card;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Machine {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub target: MachineTarget,
    #[serde(default)]
    pub status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MachineTarget {
    pub host: String,
    pub port: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MachinesResponse {
    pub machines: Vec<Machine>,
}

/// Machine status badge component
#[component]
fn StatusBadge(#[prop(into)] status: String) -> impl IntoView {
    let (bg_class, text) = match status.as_str() {
        "online" => ("bg-green-100 text-green-800", "Online"),
        "offline" => ("bg-red-100 text-red-800", "Offline"),
        "auth_failed" => ("bg-yellow-100 text-yellow-800", "Auth Failed"),
        _ => ("bg-gray-100 text-gray-800", "Unknown"),
    };

    view! {
        <span class=format!("inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {}", bg_class)>
            {text}
        </span>
    }
}

async fn fetch_machines() -> Result<Vec<Machine>, String> {
    let window = web_sys::window().ok_or("No window")?;
    let storage = window.local_storage().map_err(|_| "No storage")?.ok_or("No storage")?;
    let token = storage.get_item("np_token").map_err(|_| "No token")?;

    let mut opts = web_sys::RequestInit::new();
    opts.set_method("GET");

    let request = web_sys::Request::new_with_str_and_init("/api/machines", &opts)
        .map_err(|_| "Failed to create request")?;

    if let Some(t) = token {
        request.headers().set("Authorization", &format!("Bearer {}", t)).ok();
    }

    let resp = wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|_| "Fetch failed")?;

    let resp: web_sys::Response = resp.dyn_into().map_err(|_| "Not a response")?;
    let json = wasm_bindgen_futures::JsFuture::from(resp.json().map_err(|_| "No JSON")?)
        .await
        .map_err(|_| "JSON parse failed")?;

    let data: MachinesResponse = serde_wasm_bindgen::from_value(json)
        .map_err(|e| format!("Deserialize failed: {:?}", e))?;

    Ok(data.machines)
}

/// Machine list page
#[component]
pub fn MachineListPage() -> impl IntoView {
    let machines = LocalResource::new(|| fetch_machines());
    let machines_view = move || {
        machines.get().map(|result| {
            match &*result {
                Ok(list) => list.clone(),
                Err(_) => vec![],
            }
        }).unwrap_or_default()
    };

    view! {
        <div class="space-y-6">
            <div class="flex items-center justify-between">
                <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">
                    "Machines"
                </h1>
                <A
                    href="/machines/add"
                    attr:class="inline-flex items-center px-4 py-2 bg-indigo-600 hover:bg-indigo-700 text-white text-sm font-medium rounded-md transition-colors"
                >
                    "+ Add Machine"
                </A>
            </div>

            {move || {
                let list = machines_view();
                if list.is_empty() {
                    view! {
                        <Card>
                            <div class="text-center py-12">
                                <div class="text-gray-400 text-5xl mb-4">
                                    <svg class="w-16 h-16 mx-auto" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"/>
                                    </svg>
                                </div>
                                <h3 class="text-lg font-medium text-gray-900 dark:text-gray-100 mb-2">
                                    "No machines configured"
                                </h3>
                                <p class="text-gray-500 dark:text-gray-400 mb-4">
                                    "Add a machine to start managing NixOS systems remotely."
                                </p>
                                <A
                                    href="/machines/add"
                                    attr:class="inline-flex items-center px-4 py-2 bg-indigo-600 hover:bg-indigo-700 text-white text-sm font-medium rounded-md transition-colors"
                                >
                                    "Add Your First Machine"
                                </A>
                            </div>
                        </Card>
                    }.into_any()
                } else {
                    view! {
                        <div class="bg-white dark:bg-gray-800 shadow overflow-hidden rounded-lg">
                            <table class="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
                                <thead class="bg-gray-50 dark:bg-gray-900">
                                    <tr>
                                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                                            "Name"
                                        </th>
                                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                                            "Host"
                                        </th>
                                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                                            "Status"
                                        </th>
                                        <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                                            "Actions"
                                        </th>
                                    </tr>
                                </thead>
                                <tbody class="bg-white dark:bg-gray-800 divide-y divide-gray-200 dark:divide-gray-700">
                                    {list.into_iter().map(|machine| {
                                        let id = machine.id.clone();
                                        view! {
                                            <tr>
                                                <td class="px-6 py-4 whitespace-nowrap">
                                                    <div class="text-sm font-medium text-gray-900 dark:text-gray-100">
                                                        {machine.name}
                                                    </div>
                                                    <div class="text-sm text-gray-500 dark:text-gray-400">
                                                        {machine.description.unwrap_or_default()}
                                                    </div>
                                                </td>
                                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
                                                    {format!("{}:{}", machine.target.host, machine.target.port)}
                                                </td>
                                                <td class="px-6 py-4 whitespace-nowrap">
                                                    <StatusBadge status=machine.status />
                                                </td>
                                                <td class="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                                                    <A
                                                        href=format!("/machines/{}", id)
                                                        attr:class="text-indigo-600 hover:text-indigo-900 dark:text-indigo-400"
                                                    >
                                                        "View"
                                                    </A>
                                                </td>
                                            </tr>
                                        }
                                    }).collect::<Vec<_>>()}
                                </tbody>
                            </table>
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}
