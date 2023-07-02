use srs::{Route, Server};

fn main() {
    let mut server = Server::new();
    let mut home_route = Route::new("/");
    home_route
        .get(|req, mut res| {
            println!("got get {:?}", req.query);
            res.status(200).send("hello get request");
        })
        .post(|req, mut res| {
            println!("got post {:?}", req.body);
            res.status(200).send("hello post request");
        });

    server.use_route(home_route);

    server.listen("localhost:3000");
}
