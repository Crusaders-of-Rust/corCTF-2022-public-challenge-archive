import os
import json
import subprocess
from flask import Flask, send_from_directory, request

app = Flask(__name__, static_folder='fe')


@app.route('/api/generate', methods=['POST'])
def generate_msfrog():
    # Check if we received JSON
    if not request.is_json:
        return "I only speak JSON :c", 400

    # Grab all the msfrog accessoires
    accessoires = None
    try:
        accessoires = request.get_json()
    except:
        return "nice json", 400
        
    if not accessoires or not isinstance(accessoires, list):
        return ":msfrog:", 400

    composites = []
    for accessoire in accessoires:
        if 'type' not in accessoire:
            return "missing type lmao", 400
        
        type = accessoire['type']
        pos = accessoire.get('pos', None)
        if not pos or not isinstance(pos, dict):
            return "Ehh I need the position to supply to imagemagick", 400

        x = pos.get('x', None)
        y = pos.get('y', None)

        if not isinstance(type, str):
            return "missing type lmao", 400

        # Anti haxxor check
        if not os.path.exists("./img/" + os.path.basename(type)):
            return "I wont pass a non existing image to a shell command lol", 400

        if x is None or y is None:
            return "Ehh I need the position to supply to imagemagick", 400

        composites.append(f"img/{type} -geometry +{x}+{y} -composite")

    try:
        result = subprocess.run(
            f"convert img/base.png {' '.join(composites)} -trim png:- | base64 -w0", capture_output=True, shell=True)
        if result.returncode != 0 or len(result.stdout) == 0:
            return f"Something went wrong :\n{result.stderr}", 500
        return json.dumps({"msfrog": result.stdout.decode('utf8')})
    except:
        return json.dumps({"msfrog": "error"})


@ app.route('/')
def serve_index():
    return send_from_directory(app.static_folder, 'index.html')


@ app.route('/<path:path>')
def serve_react(path):
    if path != "" and os.path.exists(app.static_folder + '/' + path):
        return send_from_directory(app.static_folder, path)
    else:
        return "File not found", 404


if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000, threaded=True)
