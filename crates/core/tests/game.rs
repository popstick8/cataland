use cataland_core::{
    Mode, Seat,
    cities::Track,
    development::DEVELOPMENT,
    economy::{CITY, ROAD, SETTLEMENT},
    game::{Action, BuildingKind, Effect, Game, Stage},
};

#[test]
fn complete_games_conserve_resources() {
    for mode in [Mode::Base, Mode::Cities] {
        for (seed, count) in [(0, 3), (1, 4), (2, 5), (3, 6), (4, 2)] {
            fastrand::seed(seed);
            let seats: Vec<_> = (0..count)
                .map(|color| Seat {
                    name: format!("Player {color}"),
                    color,
                    connected: true,
                    ready: true,
                })
                .collect();
            let mut game = Game::new(mode, &seats, Some(0)).unwrap();
            for _ in 0..5000 {
                if game.winner.is_some() {
                    break;
                }
                let player = game
                    .pending
                    .front()
                    .map_or(game.turn.player, Effect::player);
                let hand = game.players[player].hand;
                let goal = if game.players[player].cities > 0
                    && game.buildings.iter().flatten().any(|building| {
                        building.player == player && building.kind == BuildingKind::Settlement
                    }) {
                    CITY
                } else if game.players[player].settlements > 0
                    && (0..game.board.vertices.len())
                        .any(|vertex| game.can_settle(player, vertex, false))
                {
                    SETTLEMENT
                } else if mode == Mode::Cities
                    && game.players[player].roads == 0
                    && let Some(track) = Track::ALL
                        .into_iter()
                        .find(|track| game.players[player].upgrades[track.index()] < 5)
                {
                    game.improvement_cost(player, track, 0)
                } else if !game.dev_deck.is_empty() {
                    DEVELOPMENT
                } else {
                    ROAD
                };
                let action = if let Some(prompt) = game.prompt(Some(player)) {
                    if let Some(selection) = prompt.cards {
                        let mut cards = [0; 8];
                        let discard = matches!(game.pending.front(), Some(Effect::Discard { .. }));
                        for _ in 0..selection.count {
                            let index = (0..8)
                                .filter(|&i| cards[i] < selection.available[i])
                                .max_by_key(|&i| {
                                    if discard {
                                        i32::from(hand[i])
                                            - i32::from(cards[i])
                                            - i32::from(goal[i])
                                    } else {
                                        i32::from(goal[i])
                                            - i32::from(hand[i])
                                            - i32::from(cards[i])
                                    }
                                })
                                .unwrap();
                            cards[index] += 1;
                        }
                        Action::SelectCards { cards }
                    } else if prompt.choices.is_empty() && prompt.can_skip {
                        Action::Skip
                    } else {
                        Action::Pick {
                            value: prompt.choices[fastrand::usize(..prompt.choices.len())].value,
                        }
                    }
                } else {
                    let actions = game.actions(player);
                    if matches!(game.stage, Stage::Setup { .. }) {
                        actions[fastrand::usize(..actions.len())].action.clone()
                    } else {
                        actions
                            .iter()
                            .filter_map(|choice| {
                                let score = match choice.action {
                                    Action::Roll => 9,
                                    Action::PlayCard { .. } | Action::PlayProgress { .. } => 8,
                                    Action::ActivateKnight { .. } | Action::Improve { .. } => 8,
                                    Action::RecruitKnight { .. } | Action::PromoteKnight { .. }
                                        if game.cities.as_ref().is_some_and(|cities| {
                                            cities
                                                .knights
                                                .iter()
                                                .flatten()
                                                .filter(|knight| knight.player == player)
                                                .map(|knight| knight.level)
                                                .sum::<u8>()
                                                < 3
                                        }) =>
                                    {
                                        8
                                    }
                                    Action::BuildCity { .. } => 7,
                                    Action::BuildSettlement { .. } => 6,
                                    Action::BuyDevelopment => 5,
                                    Action::BuildRoad { .. } => 4,
                                    _ => return None,
                                };
                                Some((score, choice))
                            })
                            .max_by_key(|(score, _)| *score)
                            .map(|(_, choice)| choice)
                            .or_else(|| {
                                actions.iter().find(|choice| {
                                    if let Action::BankTrade { give, take } = choice.action {
                                        hand[take.index()] < goal[take.index()]
                                            && hand[give.index()]
                                                >= goal[give.index()] + choice.cost[give.index()]
                                    } else {
                                        false
                                    }
                                })
                            })
                            .unwrap_or_else(|| {
                                actions
                                    .iter()
                                    .find(|choice| matches!(choice.action, Action::EndTurn))
                                    .unwrap()
                            })
                            .action
                            .clone()
                    }
                };
                game.apply(player, action.clone())
                    .unwrap_or_else(|error| panic!("{action:?}: {error}"));
                if count == 2 {
                    assert_eq!(
                        game.tokens + game.players.iter().map(|player| player.tokens).sum::<u8>(),
                        20
                    );
                }
                for resource in 0..8 {
                    let total = game.bank[resource]
                        + game
                            .players
                            .iter()
                            .map(|player| player.hand[resource])
                            .sum::<u16>();
                    assert_eq!(
                        total,
                        if resource < 5 {
                            if count >= 5 { 24 } else { 19 }
                        } else if mode == Mode::Cities {
                            if count >= 5 { 18 } else { 12 }
                        } else {
                            0
                        }
                    );
                }
            }
            assert!(
                game.winner.is_some(),
                "No winner in {mode:?} for seed {seed}: players={:?}, awards={:?}, bank={:?}, turn={:?}, stage={:?}, deck={}",
                game.players,
                game.awards,
                game.bank,
                game.turn,
                game.stage,
                game.dev_deck.len()
            );
        }
    }
}
