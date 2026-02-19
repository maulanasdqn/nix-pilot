use leptos::prelude::*;
use leptos_router::components::A;

use crate::components::common::Card;

/// Add machine page with form
#[component]
pub fn AddMachinePage() -> impl IntoView {
    // Form state
    let (name, set_name) = signal(String::new());
    let (host, set_host) = signal(String::new());
    let (port, set_port) = signal("22".to_string());
    let (username, set_username) = signal(String::new());
    let (auth_method, set_auth_method) = signal("agent".to_string());
    let (key_path, set_key_path) = signal(String::new());
    let (description, set_description) = signal(String::new());

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        // In a real app, this would call the API to create the machine
        // and then navigate to the machine list or detail page
        let _ = name.get(); // Use the value
    };

    view! {
        <div class="max-w-2xl mx-auto space-y-6">
            <div class="flex items-center justify-between">
                <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">
                    "Add Machine"
                </h1>
                <A
                    href="/machines"
                    attr:class="text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
                >
                    "Cancel"
                </A>
            </div>

            <Card>
                <form on:submit=on_submit class="space-y-6">
                    // Machine Name
                    <div>
                        <label for="name" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                            "Name"
                        </label>
                        <input
                            type="text"
                            id="name"
                            required=true
                            class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white sm:text-sm"
                            placeholder="my-server"
                            prop:value=name
                            on:input=move |ev| set_name.set(event_target_value(&ev))
                        />
                        <p class="mt-1 text-sm text-gray-500">
                            "A friendly name to identify this machine"
                        </p>
                    </div>

                    // Host
                    <div>
                        <label for="host" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                            "Host"
                        </label>
                        <input
                            type="text"
                            id="host"
                            required=true
                            class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white sm:text-sm"
                            placeholder="192.168.1.100 or hostname.example.com"
                            prop:value=host
                            on:input=move |ev| set_host.set(event_target_value(&ev))
                        />
                    </div>

                    // Port
                    <div>
                        <label for="port" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                            "Port"
                        </label>
                        <input
                            type="number"
                            id="port"
                            required=true
                            min="1"
                            max="65535"
                            class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white sm:text-sm"
                            prop:value=port
                            on:input=move |ev| set_port.set(event_target_value(&ev))
                        />
                    </div>

                    // Username
                    <div>
                        <label for="username" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                            "Username"
                        </label>
                        <input
                            type="text"
                            id="username"
                            required=true
                            class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white sm:text-sm"
                            placeholder="root"
                            prop:value=username
                            on:input=move |ev| set_username.set(event_target_value(&ev))
                        />
                    </div>

                    // Auth Method
                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                            "Authentication Method"
                        </label>
                        <div class="mt-2 space-y-2">
                            <label class="inline-flex items-center">
                                <input
                                    type="radio"
                                    name="auth_method"
                                    value="agent"
                                    checked=move || auth_method.get() == "agent"
                                    on:change=move |_| set_auth_method.set("agent".to_string())
                                    class="form-radio text-indigo-600"
                                />
                                <span class="ml-2 text-gray-700 dark:text-gray-300">"SSH Agent"</span>
                            </label>
                            <br/>
                            <label class="inline-flex items-center">
                                <input
                                    type="radio"
                                    name="auth_method"
                                    value="key_file"
                                    checked=move || auth_method.get() == "key_file"
                                    on:change=move |_| set_auth_method.set("key_file".to_string())
                                    class="form-radio text-indigo-600"
                                />
                                <span class="ml-2 text-gray-700 dark:text-gray-300">"Key File"</span>
                            </label>
                        </div>
                    </div>

                    // Key Path (shown only when key_file is selected)
                    <Show when=move || auth_method.get() == "key_file">
                        <div>
                            <label for="key_path" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                                "Key File Path"
                            </label>
                            <input
                                type="text"
                                id="key_path"
                                class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white sm:text-sm"
                                placeholder="~/.ssh/id_ed25519"
                                prop:value=key_path
                                on:input=move |ev| set_key_path.set(event_target_value(&ev))
                            />
                        </div>
                    </Show>

                    // Description
                    <div>
                        <label for="description" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                            "Description (optional)"
                        </label>
                        <textarea
                            id="description"
                            rows="3"
                            class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white sm:text-sm"
                            placeholder="A brief description of this machine..."
                            prop:value=description
                            on:input=move |ev| set_description.set(event_target_value(&ev))
                        />
                    </div>

                    // Submit Button
                    <div class="flex justify-end space-x-3">
                        <A
                            href="/machines"
                            attr:class="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50 dark:bg-gray-700 dark:text-gray-300 dark:border-gray-600 dark:hover:bg-gray-600"
                        >
                            "Cancel"
                        </A>
                        <button
                            type="submit"
                            class="px-4 py-2 text-sm font-medium text-white bg-indigo-600 border border-transparent rounded-md hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
                        >
                            "Add Machine"
                        </button>
                    </div>
                </form>
            </Card>
        </div>
    }
}
