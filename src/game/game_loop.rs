use game::{Player, GameBoard};
use card::CardPool;
use game::Zone;
use game::ZoneCollection;
use game::zones::Location;
use std::cell::RefCell;

pub fn run(mut pool: CardPool, mut board: GameBoard)
{
    println!("\n\nRunning core game loop. [ press Ctrl-C to exit ]\n");

    setup_decks(&pool, &mut board);

    board.shuffle_decks();

    board.run_mulligan();
    
    //println!("\n\nCardPool:  {}", serde_json::to_string(&pool).unwrap());
    //println!("\n\nLocale:    {:?}", &locale);
    //println!("\n\nGameBoard: {:?}", serde_json::to_string(&board).unwrap());

    loop
    {
        info!("Start Player #1's turn.");
        board.player1.do_turn();
        info!("Start Player #2's turn.");
        board.player2.do_turn();
    }
}
use std::rc::Rc;

fn setup_decks(pool : &CardPool, board: &mut GameBoard)
{
    let cards_to_add = vec!("auto_gen_card_000", "auto_gen_card_001", "auto_gen_card_002", 
                            "auto_gen_card_003", "auto_gen_card_004", "auto_gen_card_005", 
                            "auto_gen_card_006", "auto_gen_card_007", "auto_gen_card_008",
                            "auto_gen_card_009");

    for add in cards_to_add
    {
        let c = pool.all_cards.get(add).unwrap();//TODO remove unwrap.

        board.player1.zones.deck.add_card(RefCell::new(c.clone()),Location::Top);
        board.player2.zones.deck.add_card(RefCell::new(c.clone()),Location::Top);
    }
}
