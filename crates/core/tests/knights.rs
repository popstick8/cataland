use cataland_core::{
    Mode, Seat,
    cities::Track,
    game::{Game, Stage},
};

#[test]
fn displacement_uses_each_owners_road_network() {
    let seats: Vec<_> = (0..3)
        .map(|color| Seat {
            name: format!("Player {color}"),
            color,
            connected: true,
            ready: true,
        })
        .collect();
    let mut game = Game::new(Mode::Cities, &seats, Some(0)).unwrap();
    game.stage = Stage::Action;
    game.turn.number = 1;
    let junction = game
        .board
        .vertices
        .iter()
        .position(|point| point.neighbors.len() == 3)
        .unwrap();
    let neighbors = game.board.vertices[junction].neighbors.clone();
    let from = neighbors[0];
    let escape = neighbors[1];
    let beyond = neighbors[2];
    for &edge in &game.board.vertices[junction].edges {
        game.roads[edge] = Some(if game.board.edges[edge].vertices.contains(&escape) {
            1
        } else {
            0
        });
    }
    game.place_knight(0, from, 2, true, &[0; 8]).unwrap();
    game.place_knight(1, junction, 1, false, &[0; 8]).unwrap();
    assert!(game.move_sites(0, from).contains(&junction));
    assert!(!game.move_sites(0, from).contains(&beyond));
    assert_eq!(game.longest_route(0), 1);
    let displaced = game.move_knight(0, from, junction).unwrap().unwrap();
    game.queue_displacement(displaced, junction);
    assert_eq!(game.knight(escape).unwrap().player, 1);
    assert!(!game.knight(junction).unwrap().active);
    assert_eq!(game.longest_route(0), 2);
    game.activate_knight(0, junction, &[0; 8]).unwrap();
    assert!(!game.can_act(0, junction));
    game.turn.number += 3;
    assert!(game.can_act(0, junction));
    game.place_knight(0, from, 1, false, &[0; 8]).unwrap();
    game.players[0].upgrades[Track::Politics.index()] = 3;
    assert_eq!(game.promote_knight(0, from, &[0; 8]).unwrap(), 1);
    assert!(game.promote_knight(0, from, &[0; 8]).is_err());
}
