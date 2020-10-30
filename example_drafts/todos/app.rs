use zoon::*;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use ulid::Ulid;


type TodoId = Ulid;

// ------ Routes ------

#[Route]
#[derive(Copy, Clone)]
enum Route {
    #[path("active")]
    Active,
    #[path("completed")]
    Completed,
    #[path()]
    Root,
    Unknown,
}

#[Cache]
fn route() -> Route {
    zoon::model::url().map(Route::from)
}

#[Update]
fn set_route(route: Route) {
    zoon::model::url().set(Url::from(route))
}

// ------ Filters ------

#[derive(Copy, Clone, Eq, PartialEq, EnumIter)]
enum Filter {
    All,
    Active,
    Completed,
}

#[Model]
fn filters() -> Vec<Filter> {
    Filter::iter().collect()
}

#[Cache]
fn selected_filter() -> Filter {
    match route().inner() {
        Route::Active => Filter::Active,
        Route::Completed => Filter::Completed,
        _ => Filter::All,
    }
}

// ------ SelectedTodo ------

#[Model]
fn selected_todo() -> Option<Model<Todo>> {
    None
}

#[Update]
fn select_todo(todo: Option<Model<Todo>>) {
    selected_todo().set(todo)
}

#[Model]
fn selected_todo_title() -> Option<String> {
    selected_todo().map(|todo| todo?.map(|todo| todo.title.clone()))
}

#[Update]
fn set_selected_todo_title(title: String) {
    selected_todo_title().set(title)
}

#[Update]
fn save_selected_todo() {
    if let Some(title) = selected_todo_title().map_mut(Option::take) {
        if let Some(todo) = selected_todo().map_mut(Option::take) {
            todo.update(move |todo| todo.title = title);
        }
    }
}

// ------ Todos ------

struct Todo {
    id: TodoId,
    title: String,
    completed: bool,
}

#[Model]
fn new_todo_title() -> String {
    String::new
}

#[Update]
fn set_new_todo_title(title: String) {
    new_todo_title().set(title);
}

#[Update]
fn create_todo(title: &str) {
    let title = title.trim();
    if title.is_empty() {
        return;
    }

    let mut todo = new_model(|| Todo {
        id: TodoId::new(),
        title,
        completed: false,
    });

    todos().update(|todos| todos.push(todo));
    new_todo_title().update(String::clear);
}

#[Update]
fn remove_todo(todo: Model<Todo>) {
    if Some(todo) = selected_todo() {
        selected_todo().set(None);
    }
    todos().update(|todos| {
        if let Some(position) = todos.iter().position(|t| t == todo) {
            todos.remove(position);
        }
    });
    todo.remove();
}

#[Update]
fn toggle_todo(todo: Model<Todo>) {
    todo.update(|todo| todo.checked = !todo.checked);
}

// -- all --

#[Model]
fn todos() -> Vec<Todo> {
    Vec::new
}

#[Update]
fn check_or_uncheck_all(checked: bool) {
    if are_all_completed().inner() {
        todos().update_ref(|todos| todos.iter().for_each(toggle_todo));
    } else {
        active_todos().update_ref(|todos| todos.iter().for_each(toggle_todo));
    }
}

#[Cache]
fn todos_count() -> usize {
    todos().map(Vec::len)
}

#[Cache]
fn todos_exist() -> bool {
    todos_count().inner() != 0
}

// -- completed --

#[Cache]
fn completed_todos() -> Vec<Todo> {
    todos().map(|todos| {
        todos
            .iter()
            .filter(|todo| todo.map(|todo| todo.completed))
            .collect()
    })
}

#[Update]
fn remove_completed() {
    completed_todos().update_ref(|todos| todos.iter().for_each(remove_todo));
}

#[Cache]
fn completed_count() -> usize {
    completed_todos().map(Vec::len)
}

#[Cache]
fn completed_exist() -> bool {
    completed_count().inner() != 0
}

#[Cache]
fn are_all_completed() -> bool {
    todos_count().inner() == completed_count().inner()
}

// -- active --

#[Cache]
fn active_todos() -> Vec<Todo> {
    todos().map(|todos| {
        todos
            .iter()
            .filter(|todo| todo.map(|todo| !todo.completed))
            .collect()
    })
}

#[Cache]
fn active_count() -> usize {
    active_todos().map(Vec::len)
}

// -- filtered --

#[Cache]
fn filtered_todos() -> Cache<Vec<Todo>> {
    match app::selected_filter().inner() {
        Filter::All => app::todos().to_cache(),
        Filter::Active => app::active_todos(),
        Filter::Completed => app::completed_todos(),
    }
}
