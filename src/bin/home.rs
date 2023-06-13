use rocket_dyn_templates::{Template, context};
use rocket::{fs::FileServer, response::status::BadRequest};
use rocket::form::Form;
use rusqlite::{params, Connection, Result};
use rand::Rng;
use rocket::response::Redirect;
use lazy_static::lazy_static;
use std::sync::Mutex;
use rocket::serde::json::json;
use rusqlite::ToSql;
use urlencoding::encode;

#[macro_use]
extern crate rocket;

lazy_static! {
    static ref DB_CONN: Mutex<Connection> = Mutex::new(Connection::open("data/fpdb.db")
        .expect("Failed to open database connection"));
    static ref DECKDB_CONN: Mutex<Connection> = Mutex::new(Connection::open("data/deckdb.db")
        .expect("Failed to open database connection"));
}


// User Status Checking *******************************************
pub struct NewUser {
    username: String,
    password: String,
}

fn check_id_exists(conn: &Connection, id: i32) -> Result<bool> {
    let query = "SELECT COUNT(*) FROM users WHERE id = ?";
    let count: i64 = conn.query_row(query, [id], |row| row.get(0))?;
    Ok(count > 0)
}
fn check_if_user_exists(conn: &Connection, user: String, pass: String) -> Result<bool> {
    let query = "SELECT COUNT(*) FROM users WHERE username = ? AND password = ?";
    let count: i64 = conn.query_row(query, [&user, &pass], |row| row.get(0))?;
    Ok(count > 0)
}
fn attempt_add_user(conn: &Connection, user: String) -> Result<bool> {
    let query = "SELECT COUNT(*) FROM users WHERE username = ?";
    let count: i64 = conn.query_row(query, [&user], |row| row.get(0))?;
    Ok(count > 0)
}

impl NewUser {
    pub fn add(username: &str, password: &str) -> Result<()> {
        let conn: Connection = Connection::open("data/fpdb.db")?;

        let mut rng = rand::thread_rng();
        let mut id: i32;

        loop {
            id = rng.gen_range(0..1000);

            if check_id_exists(&conn, id)? {
                println!("ID already exists. Please enter a unique ID.");
            } else {
                break;
            }
        }

        let new_user = NewUser {
            username: username.to_owned(),
            password: password.to_owned(),
        };

        conn.execute(
            "INSERT INTO users(id, username, password) VALUES(?1, ?2, ?3)",
            params![id, new_user.username, new_user.password],
        )?;

        Ok(())
    }
}

// User Status Checking ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

// CHECKING FOR CORRECT ANSWERS **************************************

fn check_practice_answers(conn: &Connection, table: &str, english: &str, answer: &str) -> Result<bool> {
    let query = format!(
        "SELECT COUNT(*) FROM {}
            WHERE english = ?
            and japanese = ?
        ",
        table,
    );
    
    let count: i64 = conn.query_row(&query, [&english,&answer], |row| row.get(0))?;
    Ok(count > 0)
}


// CHECKING FOR CORRECT ANSWERS END  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^



// LOGIN/ SIGNUP ATTEMPTS ******************************************
#[derive(FromForm)]
struct LoginForm {
    username: String,
    password: String,
}

#[post("/login", data = "<form>")]
fn login(form: Form<LoginForm>) -> Redirect {
    let username = form.username.clone();
    let password = form.password.clone();

    let conn = DB_CONN.lock().unwrap();

    let user_exists = check_if_user_exists(&conn, username.clone(), password.clone())
        .unwrap_or(false);

    if user_exists {
        eprintln!("User: {} Signing in!",username.clone());
        eprintln!("User: {} Signing in!",password.clone());
        Redirect::to("/homescreen")
    }else {
        eprintln!("User: {} Doesn't Exist!",username.clone());
        Redirect::to("/login")
    }
}

#[derive(FromForm)]
struct SignUpForm {
    username: String,
    password: String,
}

#[post("/signup", data = "<form>")]
fn signup(form: Form<SignUpForm>) -> Redirect {
    let username = form.username.clone();
    let password = form.password.clone();

    let conn = DB_CONN.lock().unwrap();

    let username_exists = attempt_add_user(&conn, username.clone())
        .unwrap_or(false);

    if username_exists {
        eprintln!("Username: {} is taken!",username.clone());
        Redirect::to("/login")
        
    } else {
        match NewUser::add(&username, &password) {
            Ok(_) => {
                println!("User added: {}", username);
                eprintln!("Password Added: {}",password.clone());
                Redirect::to("/login")
            }
            Err(err) => {
                eprintln!("Failed to add user: {}", err);
                Redirect::to("/login")
            }
            }
        }
}
// LOGIN/ SIGNUP ATTEMPTS ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^



//Create Deck Form ******************************************************
fn check_if_table_exists(table_name: &str) -> Result<bool, rusqlite::Error> {
    let conn = DECKDB_CONN.lock().unwrap();
    let query = "SELECT name FROM sqlite_master WHERE type='table' AND name=?";
    let result: Result<String, rusqlite::Error> = conn.query_row(query, [table_name], |row| row.get(0));

    match result {
        Ok(name) => {
            eprintln!("Table '{}' exists", name);
            Ok(true)  // Table exists
        },
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(false),  // Table does not exist
        Err(err) => Err(err),  // Error occurred while checking
    }
}

fn add_new_deck(deck_name: &str) -> Result<(), rusqlite::Error> {
    let conn = DECKDB_CONN.lock().unwrap();
    let table_name = format!("{}", deck_name);

    conn.execute(
        &format!(
            "CREATE TABLE {} (
                id INTEGER PRIMARY KEY,
                english TEXT,
                japanese TEXT
            )",
            table_name
        ),
        [],
    )?;

    Ok(())
}


#[derive(FromForm)]
pub struct AddDeck{
    deckname : String,
}

#[post("/newdeck", data = "<form>")]
fn create_deck(form: Form<AddDeck>) -> Redirect {
    let deck_name = form.deckname.clone();

    let status = check_if_table_exists(&deck_name).unwrap_or(false);


    if status{
        println!("Deck Name: {} is Taken", &deck_name);
        Redirect::to("/createdeck")
    } else {
        let _ = add_new_deck(&deck_name);
        eprintln!("{} Deck Added!", &deck_name);
        Redirect::to("/createdeck")
    }
}
//Create Deck Form END ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^



// COUNTING ROWS FOR PRACTICE DECKS ***************************************

fn count_rows(conn: &Connection, table: &str) -> Result<i64> {
    let query = format!("SELECT COUNT(*) FROM {}",table);
    let count: Option<i64> = conn.query_row(&query, [], |row| row.get(0))?;
    Ok(count.unwrap_or(0))
}


// END COUNDING ROWS FOR PRACTICE DECKS ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^



// POSTS Redirect Templates  **********************************************
#[post("/library")]
fn library_redirect() -> Redirect {
    Redirect::to("/library")
}

#[derive(FromForm)]
pub struct RemoveDeck {
    table: String,
}

#[post("/remove-deck", data = "<form>")]
fn remove_deck(form: Form<RemoveDeck>) -> Redirect {
    let table_name = form.table.clone();

    let conn = DECKDB_CONN.lock().unwrap();

    match conn.execute(
        &format!(
            "DROP TABLE {}",
            table_name
        ),
        [],
    ) {
        Ok(_) => Redirect::to("/library"),
        Err(e) => {
            eprintln!("Failed to drop table: {}", e);
            Redirect::to("/error")
        }
    }
}

#[post("/create-deck")]
fn create_deck_redirect() -> Redirect {
    Redirect::to("/createdeck")
}

#[post("/sign-out")]
fn sign_out() -> Redirect {
    Redirect::to("/login")
}

#[derive(FromForm)]
pub struct AddToDeck {
    table: String,
}

#[post("/addto-deck", data = "<form>")]
fn add_to_deck(form: Form<AddToDeck>) -> Redirect {
    let table_name = form.table.clone();

    eprintln!("Table name is {}, rerouting to addtodeck.html", &table_name);

    Redirect::to(format!("/addtodeck/{}", &table_name))
}

#[derive(FromForm)]
pub struct AddNotes {
    english: String,
    japanese: String,
    table: String,
}

#[post("/add", data = "<form>")]
fn add_note_to_deck(form: Form<AddNotes>) -> Redirect {
    let eng_word = form.english.clone();
    let jap_word = form.japanese.clone();
    let table_name: String = form.table.clone();

    let conn = DECKDB_CONN.lock().unwrap();

    let count: i64= count_rows(&conn, &table_name).unwrap();

    let query = format!(
        "INSERT INTO {} (id, english, japanese) VALUES (?,?,?)",
        table_name
    );

    let params: &[&(dyn ToSql)] = &[&count, &eng_word, &jap_word];
    conn.execute(&query, params).expect("Failed to insert Values");


    eprintln!("ID ... is, English Word is {}, Japanese Word is {}, Table name is {}", &eng_word, &jap_word, &table_name);
    Redirect::to(format!("/addtodeck/{}", &table_name))

}


// WILL USE TO UPDATE PRACTICE TEMPLATE URL
fn get_next_english_word(conn: &Connection, table: &str, current_word: &str) -> Result<String, BadRequest<String>> {

    let id_query = format!(
        "SELECT id from {} where
            english = ?",
            table
    );
    

    let current_id: Result<i64, rusqlite::Error> = conn.query_row(&id_query, [current_word], |row| row.get(0));
    let num_rows = count_rows(&conn, &table);
    if let Ok(current_id) = current_id {
        if current_id >= num_rows.unwrap() {
            return Err(BadRequest(Some("This is the final word!".to_string())));
        }
    } else {
        return Err(BadRequest(Some("Failed to retrieve current_id".to_string())));
    }

    let next_id = current_id.unwrap() + 1;

    let word_query = format!(
        "SELECT english from {} where
            id = ?",
            table
    );

    let next_word: Result<String, rusqlite::Error> = conn.query_row(&word_query, [&next_id], |row| row.get(0));

    match next_word {
        Ok(word) => Ok(word),
        Err(_) => Err(BadRequest(Some("Failed to retrieve next word".to_string()))),
    }
}

#[derive(FromForm)]
pub struct PracticeDeck {
    table: String,
}

#[post("/practice-deck", data = "<form>")]
fn practice_deck(form: Form<PracticeDeck>) -> Redirect {
    let table_name = form.table.clone();

    eprintln!("Table name is {}, rerouting to addtodeck.html", &table_name);

    let conn = DECKDB_CONN.lock().unwrap();

    let query = format!(
        "SELECT english from {} where
            id = 0",
        table_name
    );

    let english_word: String = match conn.query_row(&query, [], |row| row.get(0)) {
        Ok(word) => word,
        Err(err) => {
            eprintln!("Failed to retrieve English word: {}", err);
            return Redirect::to("/library");
        }
    };

    let encoded_english_word = encode(&english_word);
    let encoded_url = format!("/practice/{}/{}", &table_name, &encoded_english_word);

    Redirect::to(encoded_url)
}


#[derive(FromForm)]
pub struct Answers {
    table: String,
    answer: String,
    english_word: String,
}

#[post("/check-answer", data = "<form>")]
fn check_answer(form: Form<Answers>) -> Redirect {
    let table_name = form.table.clone();
    let jap_word = form.answer.clone();
    let eng_word = form.english_word.clone();

    eprintln!("English Word {}, Jap Answer: {} from Table: {}", &eng_word, &jap_word, &table_name);

    let conn = DECKDB_CONN.lock().unwrap();

    let query: bool = check_practice_answers(&conn, &table_name, &eng_word, &jap_word).unwrap_or(false);

    if query {
        let is_last_word = format!(
            "select id from {} where
                english = ?",
            table_name
        );

        let num_rows = count_rows(&conn, &table_name).unwrap_or(0);

        let curr_id: Result<i64, rusqlite::Error> = conn.query_row(&is_last_word, [&eng_word], |row| row.get(0));
        if (curr_id.unwrap()) + 1 == num_rows {
            eprintln!("Your Answer {} was correct!", &jap_word);
            Redirect::to("/library")
        } else {
            eprintln!("Your Answer {} was correct!", &jap_word);
            let next_english_word = get_next_english_word(&conn, &table_name, &eng_word).unwrap();
            let encoded_table_name = encode(&table_name);
            let encoded_next_english_word = encode(&next_english_word);
            Redirect::to(format!("/practice/{}/{}", encoded_table_name, encoded_next_english_word))
        }
    } else {
        eprintln!("Your Answer {} was incorrect!", &jap_word);
        let encoded_table_name = encode(&table_name);
        let encoded_english_word = encode(&eng_word);
        Redirect::to(format!("/practice/{}/{}", encoded_table_name, encoded_english_word))
    }
}

// POSTS Redirect Templates END ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^


// GETTING TABLES FOR LIBRARY *****************************************
fn get_table_names() -> Result<Vec<String>, rusqlite::Error> {
    let conn = DECKDB_CONN.lock().unwrap();
    let query = "SELECT name FROM sqlite_master WHERE type='table'";
    let mut stmt = conn.prepare(query)?;

    let table_names: Result<Vec<String>, rusqlite::Error> = stmt
        .query_map([], |row| row.get(0))
        .map(|result| result.collect())
        .unwrap();

    // Add a debug print here
    println!("Table names: {:?}", table_names);

    table_names
}
// GETTING TABELS FOR LIBRARY END ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^




// TEMPLATE RENDERING *************************************************
#[get("/")]
fn loginpage() -> Template {
    Template::render("login", context! { field: "value" })
}

#[get("/login")]
fn login_page() -> Template {
    Template::render("login", context! {})
}

#[get("/homescreen")]
fn homescreen() -> Template {
    Template::render("homescreen", context! { field: "value" })
}

#[get("/createdeck")]
fn createdeck() -> Template {
    Template::render("createdeck", context! { field: "value" })
}

#[get("/library")]
fn library() -> Template {
    let table_names = get_table_names().unwrap_or_else(|_| Vec::new());

    println!("Existing Decks Are {:?}", table_names);

    let context = json!({
        "tables": table_names,
    });
    Template::render("library", context)
}


#[get("/addtodeck/<table_name>")]
fn addtodeck(table_name: String) -> Template {
    eprintln!("Rerouting to addtodeck.html.tera");
    let context = json!({
        "table": table_name,
    });
    Template::render("addtodeck", context)
}

#[get("/practice/<table_name>/<english_word>")]
fn practice(table_name: String, english_word: String) -> Result<Template, BadRequest<String>> {
    eprintln!("Rerouting to practice.html.tera");
    let japanese_word = "asdf";
    let context = json!(
        {
            "table": table_name,
            "japaneseWord": japanese_word,
            "englishWord": english_word, 
        }
    );
    Ok(Template::render("practice", context))
    
}

// TEMPLATE RENDERING END ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^



#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(Template::fairing())
        .mount("/", routes![
            loginpage,
            login,
            login_page,
            signup,
            homescreen,
            create_deck,
            createdeck,
            sign_out,
            create_deck_redirect,
            library_redirect,
            library,
            addtodeck,
            add_to_deck,
            add_note_to_deck,
            practice,
            practice_deck,
            check_answer,
            remove_deck
        ])
        .mount("/static", FileServer::from("static"))
}

