import chess
from stockfish import Stockfish
import chess.engine
import random
import os

FLAG = os.getenv("FLAG", "test_flag")

toxic_msges = [
    "?",
    "rip bozo",
    "so bad lmfaoo",
    "ez",
    "skill issue",
    "mad cuz bad",
    "hold this L",
    "L + ratio + you fell off",
    "i bet your main category is stego",
    "have you tried alt+f4?"
]

win_msges = [
    "lmaooooooooo ur so bad",
    "was that it?",
    "zzzzzzzzzzzzzzzzzzzzzz",
    "hopefully the next game wont be so quick",
    "nice try - jk that was horrible",
    "this aint checkers man"
]

class MSFrog:
    def __init__(self, emit):
        self.emit = emit
        self.engine = Stockfish("./stockfish_14_linux_x64_avx2/stockfish_14_x64_avx2", parameters={"Threads": 4})
        self.quit = False

    def configure(self, options):
        try:
            if "Hash" in options and options["Hash"] > 128:
                return
            if "Threads" in options and options["Threads"] > 1:
                return
            if "Debug Log File" in options:
                return
            self.engine.update_engine_parameters(options)
        except Exception as e:
            print(e)
            self.emit("chat", {"name": "🐸", "msg": "Error configuring engine"})

    def chat(self, msg):
        self.emit("chat", {"name": "🐸", "msg": msg})

    def ratio(self):
        self.chat(random.choice(win_msges))

    def normalthink(self, move, board):
        if self.quit:
            return

        best_move = None
        broke = True
        while broke:
            try:
                self.engine.set_fen_position(board.fen())
                broke = False
            except Exception as e:
                print(engine)
                self.engine = Stockfish("./stockfish_14_linux_x64_avx2/stockfish_14_x64_avx2", parameters={"Threads": 4})
                broke = True
        broke = False
        while best_move is None:
            try:
                #best_move = self.engine.play(board, chess.engine.Limit(time=1.5))
                best_move = chess.Move.from_uci(self.engine.get_best_move_time(1000))
            except Exception as e:
                print(e)
                broke = True
                self.chat("uhh... my brain broke hol up - if this persists, pls open a ticket on Discord")
                self.engine = Stockfish("./stockfish_14_linux_x64_avx2/stockfish_14_x64_avx2", parameters={"Threads": 4})
                self.engine.set_fen_position(board.fen())

        if broke:
            self.chat("back online")

        if board.is_capture(best_move):
            self.chat(random.choice(toxic_msges))

        return best_move

    def resign(self):
        if self.quit:
            return
        self.chat("wtf???")
        self.chat(FLAG)
