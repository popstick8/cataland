use cataland_core::{
    Mode, Seat,
    board::Resource,
    game::{Action, Effect, Game, Stage},
    progress::Progress,
};

fn settle(game: &mut Game) {
    for _ in 0..100 {
        game.advance();
        let Some(effect) = game.pending.front() else {
            return;
        };
        let player = if matches!(effect, Effect::Discard { .. }) {
            game.pending
                .iter()
                .take_while(|effect| matches!(effect, Effect::Discard { .. }))
                .filter(|effect| matches!(effect, Effect::Discard { count, .. } if *count > 0))
                .last()
                .unwrap()
                .player()
        } else {
            effect.player()
        };
        let prompt = game.prompt(Some(player)).unwrap();
        let public = game.prompt(None).unwrap();
        assert!(public.cards.is_none() && public.choices.is_empty());
        let action = if let Some(selection) = prompt.cards {
            let mut cards = [0; 8];
            let mut remaining = selection.count;
            for (index, &available) in selection.available.iter().enumerate() {
                cards[index] = remaining.min(available);
                remaining -= cards[index];
            }
            assert_eq!(remaining, 0);
            Action::SelectCards { cards }
        } else if let Some(choice) = prompt.choices.first() {
            Action::Pick {
                value: choice.value,
            }
        } else {
            assert!(prompt.can_skip, "No choices for {effect:?}");
            Action::Skip
        };
        game.apply(player, action).unwrap();
        if !prompt.responses.is_empty()
            && let Some(next) = game.prompt(None).filter(|next| !next.responses.is_empty())
        {
            assert_eq!(next.responses.len(), prompt.responses.len());
            assert!(
                next.responses
                    .iter()
                    .any(|response| response.player == player && response.complete)
            );
            assert!(game.pending_index(player).is_none());
        }
    }
    panic!("Card resolution did not finish: {:?}", game.pending);
}

fn conservation(game: &Game) {
    for resource in Resource::ALL {
        let total = game.bank[resource.index()]
            + game
                .players
                .iter()
                .map(|player| player.hand[resource.index()])
                .sum::<u16>();
        assert_eq!(
            total,
            if resource.index() < 5 {
                if game.humans >= 5 { 24 } else { 19 }
            } else if game.humans >= 5 {
                18
            } else {
                12
            }
        );
    }
    let cities = game.cities.as_ref().unwrap();
    let cards = cities.decks.iter().map(Vec::len).sum::<usize>()
        + game
            .players
            .iter()
            .map(|player| player.progress.len() + player.revealed.len())
            .sum::<usize>();
    assert_eq!(cards, 54);
    for (owner, player) in game.players.iter().enumerate() {
        assert_eq!(
            usize::from(player.roads)
                + game
                    .roads
                    .iter()
                    .filter(|&&road| road == Some(owner))
                    .count(),
            15
        );
        assert_eq!(
            usize::from(player.cities)
                + game
                    .buildings
                    .iter()
                    .enumerate()
                    .filter(
                        |(vertex, building)| building.as_ref().is_some_and(|building| building
                            .player
                            == owner
                            && (building.kind == cataland_core::game::BuildingKind::City
                                || cities.ruins.contains(vertex)))
                    )
                    .count(),
            4
        );
    }
}

#[test]
fn progress_and_barbarians_resume_the_interrupted_turn() {
    for count in [2, 4, 6] {
        fastrand::seed(count as u64);
        let seats: Vec<_> = (0..count)
            .map(|color| Seat {
                name: format!("Player {color}"),
                color,
                connected: true,
                ready: true,
            })
            .collect();
        let mut game = Game::new(Mode::Cities, &seats, Some(0)).unwrap();
        while matches!(game.stage, Stage::Setup { .. }) {
            let player = game.turn.player;
            let action = game.actions(player)[0].action.clone();
            game.apply(player, action).unwrap();
        }
        for player in 0..count {
            for resource in Resource::ALL {
                game.take_bank(player, resource, 3);
            }
            game.players[player].upgrades = [3; 3];
            while game.knight_sites(player).is_empty() {
                let edge = (0..game.board.edges.len())
                    .find(|&edge| game.can_road(player, edge))
                    .unwrap();
                game.build_road(player, edge, &[0; 8]).unwrap();
            }
            let vertex = game.knight_sites(player)[0];
            game.place_knight(player, vertex, 1, true, &[0; 8]).unwrap();
        }
        game.barbarian_attack();
        settle(&mut game);
        assert!(game.robber.is_some());
        assert!(
            game.cities
                .as_ref()
                .unwrap()
                .knights
                .iter()
                .flatten()
                .all(|knight| !knight.active)
        );
        conservation(&game);
        for card in Progress::ALL.into_iter().filter(|card| !card.victory()) {
            game.turn.player = 0;
            game.turn.primary = 0;
            game.turn.number += 1;
            game.turn.dice.clear();
            game.stage = if card == Progress::Alchemy {
                Stage::Production
            } else {
                Stage::Action
            };
            let mut found = false;
            for deck in &mut game.cities.as_mut().unwrap().decks {
                if let Some(index) = deck.iter().position(|&held| held == card) {
                    deck.remove(index);
                    found = true;
                    break;
                }
            }
            if !found {
                for player in &mut game.players {
                    if let Some(index) = player.progress.iter().position(|&held| held == card) {
                        player.progress.remove(index);
                        found = true;
                        break;
                    }
                }
            }
            assert!(found);
            game.players[0].progress.push(card);
            game.apply(0, Action::PlayProgress { card }).unwrap();
            settle(&mut game);
            if card == Progress::CommercialHarbor {
                game.apply(0, Action::HarborTrade { target: 1 }).unwrap();
                settle(&mut game);
            }
            conservation(&game);
        }
        game.stage = Stage::Action;
        game.apply(0, Action::EndTurn).unwrap();
        settle(&mut game);
        assert!(game.players[0].progress.len() <= 4);
        game.barbarian_attack();
        settle(&mut game);
        conservation(&game);
        assert!(
            game.pending
                .iter()
                .all(|effect| !matches!(effect, Effect::Production { .. }))
        );
    }
}
