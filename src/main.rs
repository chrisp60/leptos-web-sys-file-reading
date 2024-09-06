use js_sys::Uint8Array;
use leptos::*;
use wasm_bindgen::{prelude::Closure, JsCast, UnwrapThrowExt};
use web_sys::{window, Element, FileReader, HtmlInputElement};

const INPUT_ID: &str = "fileInput";

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount_to_body(App);
}

fn get_element_id(id: &str) -> Option<Element> {
    window()
        .unwrap_throw()
        .document()
        .unwrap_throw()
        .get_element_by_id(id)
}

#[derive(Clone, Debug)]
struct Upload {
    name: String,
    content: String,
}

#[derive(Default, Debug, Clone, Copy)]
struct UploadSignal(RwSignal<Vec<Upload>>);

#[component]
fn App() -> impl IntoView {
    let signal = UploadSignal::default();
    // Just using context here because it is convenient. Passing props
    // around is fine as well.
    provide_context(signal);
    view! {
      <h1>"File Reader"</h1>
      <p>
        <ul>
          <li>
            <strong>"File must be UTF8 encoded (plain text)"</strong>
          </li>
          <li>
            <p>"Check console if something doesnt work"</p>
          </li>
        </ul>
      </p>
      <input type="file" id=INPUT_ID on:change=from_input multiple required/>
      <button on:click=move |_| {
          signal.0.update(Vec::clear)
      }>"Clear List"</button>
      <hr/>
      <Results/>
    }
}

fn from_input(_: ev::Event) {
    let input = get_element_id(INPUT_ID)
        .and_then(|e| e.dyn_into::<HtmlInputElement>().ok())
        .unwrap_throw();
    let UploadSignal(file_signal) = expect_context();
    file_signal.update(Vec::clear); // Clear list from previous change
    let files = input.files().unwrap_throw();
    let mut index = 0;

    while let Some(file) = files.get(index) {
        let reader = FileReader::new().unwrap_throw();
        let cloned = reader.clone();
        let name = file.name();

        // This closure only needs to be called a single time as we will just
        // remake it on each loop
        // * web_sys drops it for us when using this specific constructor
        let read_file = Closure::once_into_js(move |_: ev::ProgressEvent| {
            // `.result` is only valid after the `read_*` operation is complete on the
            // FileReader
            // https://developer.mozilla.org/en-US/docs/Web/API/FileReader/result
            let result = cloned.result().unwrap_throw();
            let array = Uint8Array::new(&result).to_vec();
            let content = String::from_utf8(array).unwrap_throw();
            file_signal.update(|items| items.push(Upload { name, content }))
        });
        reader.set_onloadend(Some(read_file.as_ref().unchecked_ref()));

        // read_as_array_buffer takes a &Blob
        //
        // Per https://w3c.github.io/FileAPI/#file-section
        // > A File object is a Blob object with a name attribute..
        //
        // File is a subclass (inherits) from the Blob interface, so a File
        // can be used anywhere a Blob is required.
        reader.read_as_array_buffer(&file).unwrap_throw();

        index += 1;
    }
}

#[component]
fn Results() -> impl IntoView {
    let UploadSignal(files) = expect_context();
    let body = move || {
        files
            .get()
            .into_iter()
            .map(|Upload { name, content }| {
                view! {
                  <tr>
                    <td>{name}</td>
                    <td>
                      <pre>{content}</pre>
                    </td>
                  </tr>
                }
            })
            .collect_view()
    };
    view! {
      <table>
        <thead>
          <tr>
            <th>"Name"</th>
            <th>"content"</th>
          </tr>
        </thead>
        <tbody>{body}</tbody>
      </table>
    }
}
