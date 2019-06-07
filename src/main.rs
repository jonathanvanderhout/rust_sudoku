// server related imports and functions

#[macro_use]
extern crate json;

use actix_web::{
    error, middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer,
};
use bytes::BytesMut;
use futures::{Future, Stream};
use json::JsonValue;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct MyObj {
    name: String,
    number: i32,
    thing: i32
}

#[derive(Debug, Serialize, Deserialize)]
struct BoardObject {
    name: String,
    number: i32,
}
const N: usize = 9;

#[derive(Debug, Serialize, Deserialize)]
struct GridObject{
    grid: [[usize; N]; N]
}



/// This handler uses json extractor
fn index(item: web::Json<GridObject>) -> HttpResponse {
    // println!("model: {:?}", &item);
    // let mut grid: [[usize; N]; N] = [[8,0,0,0,0,0,0,0,0],
    //                                   [0,0,3,6,0,0,0,0,0],
    //                                   [0,7,0,0,9,0,2,0,0],
    //                                   [0,5,0,0,0,7,0,0,0],
    //                                   [0,0,0,0,4,5,7,0,0],
    //                                   [0,0,0,1,0,0,0,3,0],
    //                                   [0,0,1,0,0,0,0,6,8],
    //                                   [0,0,8,5,0,0,0,1,0],
    //                                   [0,9,0,0,0,0,4,0,0]];
    let mut grid_object = item.0;
    // let mut grid = grid.grid;
    solve_sudoku(&mut grid_object.grid);
    HttpResponse::Ok().json(grid_object) // <- send response
}
/// This handler uses json extractor with limit
fn extract_item(item: web::Json<MyObj>, req: HttpRequest) -> HttpResponse {
    println!("request: {:?}", req);
    println!("model: {:?}", item);

    HttpResponse::Ok().json(item.0) // <- send json response
}

const MAX_SIZE: usize = 262_144; // max payload size is 256k

/// This handler manually load request payload and parse json object
fn index_manual(
    payload: web::Payload,
) -> impl Future<Item = HttpResponse, Error = Error> {
    // payload is a stream of Bytes objects
    payload
        // `Future::from_err` acts like `?` in that it coerces the error type from
        // the future into the final error type
        .from_err()
        // `fold` will asynchronously read each chunk of the request body and
        // call supplied closure, then it resolves to result of closure
        .fold(BytesMut::new(), move |mut body, chunk| {
            // limit max size of in-memory payload
            if (body.len() + chunk.len()) > MAX_SIZE {
                Err(error::ErrorBadRequest("overflow"))
            } else {
                body.extend_from_slice(&chunk);
                Ok(body)
            }
        })
        // `Future::and_then` can be used to merge an asynchronous workflow with a
        // synchronous workflow
        .and_then(|body| {
            // body is loaded, now we can deserialize serde-json
            let obj = serde_json::from_slice::<MyObj>(&body)?;
            Ok(HttpResponse::Ok().json(obj)) // <- send response
        })
}

/// This handler manually load request payload and parse json-rust
fn index_mjsonrust(pl: web::Payload) -> impl Future<Item = HttpResponse, Error = Error> {
    pl.concat2().from_err().and_then(|body| {
        // body is loaded, now we can deserialize json-rust
        let result = json::parse(std::str::from_utf8(&body).unwrap()); // return Result
        let injson: JsonValue = match result {
            Ok(v) => v,
            Err(e) => json::object! {"err" => e.to_string() },
        };
        Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(injson.dump()))
    })
}

// sudoku related functions
const UNASSIGNED : usize= 0;
// const N : usize= 9;

fn find_unassigned_location(grid:[[usize; N]; N],
							rowindex: &mut usize, colindex: &mut usize) -> bool
{
    for (_i, row) in grid.iter().enumerate() {
        for (_j, col) in row.iter().enumerate() {
            // print!("{}", col);
            if *col == UNASSIGNED {
                *rowindex = _i ;
                *colindex = _j ;
                return true
            }
        }
    }
    false


}

fn used_in_row(grid:[[usize; N]; N],rowindex:  usize, num:  usize) -> bool
{
    for col in 0..N {
        if grid[rowindex][col] == num{
            return true
        }
    }
    false
}

fn used_in_col(grid:[[usize; N]; N], colindex: usize, num: usize) -> bool
{
    for row in 0..N {
        if grid[row][colindex] == num{
            return true
        }
    }
    false
}

fn used_in_box(grid:[[usize; N]; N], box_start_row: usize,  box_start_col: usize,  num: usize) -> bool
{
	for row in 0..3{
        for col in 0..3 {
            if grid[row + box_start_row][col + box_start_col] == num{
                return true
            }
        }
    }
	 false
}


fn is_safe(grid:[[usize; N]; N], row:usize, col:usize, num:usize) -> bool
{
	return !used_in_row(grid, row, num) &&
		   !used_in_col(grid, col, num) &&
		   !used_in_box(grid, row - row % 3 , col - col % 3, num) &&
			grid[row][col] == UNASSIGNED
}

fn print_grid(grid:[[usize;N];N])
{
    for (_i, row) in grid.iter().enumerate() {
        for (_j, col) in row.iter().enumerate() {

            print!("{}  ", col);
        }
        println!();
        println!();
    }
    println!();

}

fn solve_sudoku(grid:&mut [[ usize; N]; N]) -> bool
{
	let mut col : usize = 0 ;
    let mut row : usize = 0 ;


	if !find_unassigned_location(*grid, & mut row, & mut col)
	   {return true} // success!


	for num in 1..(N+1){

		if is_safe(*grid, row, col, num)
		{
			grid[row][col] = num;

			if solve_sudoku( grid){
                return true

            }

			grid[row][col] = UNASSIGNED;
		}
	}
	return false
}


fn main() -> std::io::Result<()> {


    // let mut grid: [[usize; N]; N] = [[0,0,1,0,0,0,0,0,0],
    //                                  [0,0,0,0,0,0,0,0,0],
    //                                  [0,0,0,0,0,0,0,0,0],
    //                                  [0,0,0,0,0,0,0,0,0],
    //                                  [0,0,0,0,0,0,0,0,0],
    //                                  [0,0,0,0,0,0,0,0,0],
    //                                  [0,0,0,0,0,0,0,0,0],
    //                                  [0,0,0,0,0,0,0,0,0],
    //                                  [0,0,0,0,0,0,0,0,1]];

     let mut grid: [[usize; N]; N] = [[8,0,0,0,0,0,0,0,0],
                                      [0,0,3,6,0,0,0,0,0],
                                      [0,7,0,0,9,0,2,0,0],
                                      [0,5,0,0,0,7,0,0,0],
                                      [0,0,0,0,4,5,7,0,0],
                                      [0,0,0,1,0,0,0,3,0],
                                      [0,0,1,0,0,0,0,6,8],
                                      [0,0,8,5,0,0,0,1,0],
                                      [0,9,0,0,0,0,4,0,0]];

    print_grid(grid);
    if solve_sudoku(&mut grid){
        print_grid(grid);
    }
    else{
        println!("no solution");
    }

    std::env::set_var("RUST_LOG", "actix_web=info");
env_logger::init();

HttpServer::new(|| {
    App::new()
        // enable logger
        .wrap(middleware::Logger::default())
        .data(web::JsonConfig::default().limit(4096)) // <- limit size of the payload (global configuration)
        // .service(web::resource("/extractor").route(web::post().to(index)))
        // .service(
        //     web::resource("/extractor2")
        //         .data(web::JsonConfig::default().limit(1024)) // <- limit size of the payload (resource level)
        //         .route(web::post().to_async(extract_item)),
        // )
        // .service(web::resource("/manual").route(web::post().to_async(index_manual)))
        // .service(
        //     web::resource("/mjsonrust").route(web::post().to_async(index_mjsonrust)),
        // )
        .service(web::resource("/").route(web::post().to(index)))
})
.bind("127.0.0.1:8080")?
.run()

}
