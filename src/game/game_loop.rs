use game::{Player, GameBoard};
use card::CardPool;

pub fn run<P1: Player, P2: Player>(mut pool: CardPool, mut board: GameBoard<P1,P2>)
{
    println!("\n\nRunning core game loop. [ press Ctrl-C to exit ]\n");

    setup_decks(&pool, &mut board);

    board.shuffle_decks();
    
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

fn setup_decks<P1: Player, P2: Player>(pool : &CardPool, board: &mut GameBoard<P1,P2>)
{
    let cards_to_add = vec!("auto_gen_card_000", "auto_gen_card_001", "auto_gen_card_002", 
                            "auto_gen_card_003", "auto_gen_card_004", "auto_gen_card_005", 
                            "auto_gen_card_006", "auto_gen_card_007", "auto_gen_card_008",
                            "auto_gen_card_009");

    for add in cards_to_add
    {
        let c = pool.all_cards.get(add).unwrap();//TODO remove unwrap.

        board.p1_zones.deck.push(Box::new(c.clone()));
        board.p2_zones.deck.push(Box::new(c.clone()));
    }
}
