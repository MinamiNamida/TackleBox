use std::{collections::HashMap, time::Duration};
use rand::prelude::*;
use tokio::time::timeout;
use uuid::Uuid;

//--------------------------------------------------
// 1. 基础数据类型（牌、玩家ID、阶段）
//--------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Suit { Clubs, Diamonds, Hearts, Spades }

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Rank { Two, Three, Four, Five, Six, Seven, Eight, Nine, Ten, Jack, Queen, King, Ace }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Card { pub rank: Rank, pub suit: Suit }

pub type PlayerId = uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage { Waiting, PreFlop, Flop, Turn, River, Showdown, Finished }

//--------------------------------------------------
// 2. 玩家与牌堆
//--------------------------------------------------


pub struct Player {
    pub id: PlayerId,
    pub name: String,
    pub chips: u32,
    pub hand: Option<(Card, Card)>,
    pub active: bool,
    pub current_bet: u32,
}

pub struct Deck { cards: Vec<Card> }
impl Deck {
    pub fn new() -> Self { 
        let mut cards = Vec::with_capacity(52);
        for &suit in &[Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades] {
            for &rank in &[
                Rank::Two, Rank::Three, Rank::Four, Rank::Five, Rank::Six,
                Rank::Seven, Rank::Eight, Rank::Nine, Rank::Ten,
                Rank::Jack, Rank::Queen, Rank::King, Rank::Ace,
            ] {
                cards.push(Card { suit, rank });
            }
        }
        Self { cards }        
    }
    pub fn shuffle(&mut self) {
        let mut rng = rand::rng();
        self.cards.shuffle(&mut rng);
    }
    pub fn deal(&mut self) -> Option<Card> { self.cards.pop() }
}


//--------------------------------------------------
// 3. 游戏状态
//--------------------------------------------------
pub struct PokerGame {
    pub players: Vec<Player>,
    pub deck: Deck,
    pub board: Vec<Card>,
    pub stage: Stage,
    pub pot: u32,
}

impl PokerGame {
    pub fn new() -> Self { /* 初始化 */ }
    pub fn deal_hole_cards(&mut self) { /* 发底牌 */ }
    pub fn deal_flop(&mut self) { /* 发3张 */ }
    pub fn deal_turn(&mut self) { /* 发1张 */ }
    pub fn deal_river(&mut self) { /* 发1张 */ }
    pub fn next_stage(&mut self) { /* 阶段迁移 */ }
}

