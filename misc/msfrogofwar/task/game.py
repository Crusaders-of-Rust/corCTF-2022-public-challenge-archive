import chess
from stockfish import Stockfish
import eventlet

from enemy import MSFrog

games = {}

class Game:
    def __init__(self, emit):
        self.board = chess.Board(chess.STARTING_FEN)
        self.num_moves = 0
        self.emit = emit
        self.msfrog = MSFrog(emit)

    def get_moves(self):
        return self.get_side_moves(self.get_turn())

    def get_side_moves(self, side):
        new_board = self.board.copy()
        new_board.turn = side
        moves = new_board.legal_moves
        return [m.uci() for m in moves]

    def is_player_turn(self):
        return self.board.turn == chess.WHITE

    def get_player_state(self):
        return {
            "pos": self.board.fen(),
            "moves": self.get_side_moves(chess.WHITE),
            "your_turn": self.is_player_turn(),
            "game_over": self.board.is_game_over()
        }

    def play_move(self, move):
        if self.board.is_game_over() or not move in self.board.legal_moves:
            return False

        self.board.push(move)
        return True

    def player_move(self, data):
        if self.board.turn != chess.WHITE or self.board.is_game_over():
            return

        if type(data) is not str:
            return

        if not self.play_move(chess.Move.from_uci(data)):
            self.emit("chat", {"name": "System", "msg": "Invalid move"})
            return

        self.emit('state', self.get_player_state())  
        eventlet.sleep(0)

        self.num_moves += 1
        if self.num_moves > 30:
            self.emit("chat", {"name": "🐸", "msg": "You took too long L"})
            return

        if self.board.outcome() is not None:
            if self.board.outcome().winner == chess.WHITE:
                self.msfrog.resign()
            else:
                self.msfrog.ratio()
            return

        enemy_move = self.msfrog.normalthink(data, self.board)
        self.play_move(enemy_move)

        self.emit('state', self.get_player_state())

        if self.board.outcome() is not None:
            if self.board.outcome().winner == chess.WHITE:
                self.msfrog.resign()
            else:
                self.msfrog.ratio()
            return

def start(id, emit):
    games[id] = Game(emit)
    return games[id]

def destroy(id):
    if id in games:
        del games[id]

def get(id):
    return games[id]