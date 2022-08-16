from flask import Flask, render_template, request
from flask_socketio import SocketIO, emit

import game

app = Flask(__name__, static_url_path='', static_folder='static')
app.config['SECRET_KEY'] = 'secret!'
socketio = SocketIO(app, cors_allowed_origins=['https://msfrogofwar.be.ax', 'http://localhost'])

@app.after_request
def add_header(response):
    response.headers['Cache-Control'] = 'max-age=604800'
    return response

@app.route('/')
def index_route():
    return app.send_static_file('index.html')

@socketio.on('connect')
def on_connect(args):
    game.start(request.sid, emit)
    emit('state', game.get(request.sid).get_player_state())

@socketio.on('disconnect')
def on_disconnect():
    game.destroy(request.sid)

@socketio.on('move')
def onmsg_move(move):
    game.get(request.sid).player_move(move)

@socketio.on('options')
def engine_options(options):
    game.get(request.sid).msfrog.configure(options)
