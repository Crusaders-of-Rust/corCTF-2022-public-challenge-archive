from flask import Flask, render_template, request, redirect, flash, Markup
from encryption import *
import sys, os


LICENSE = "NtymTiCisQnP4O8ZHZ7KVKqELMRHbsCTnx/4/1JGz8yaPrV00/+Gl6/oqOpCHh+R"
FLAG = "yapUR/sz4ghNmITpaSdzG1EloJOTUY5p32Sp1VY9JG/LDv1dq3qGvxW61jxOvemjWlDwoNEfrBw+R+0s074UYQ=="
PROMO_CODE_HASH = '8396df15b621c87e2dd9c0afcdee40404ca1ac179e7d4a595a3eb60edf52afcd'
LICENSE_HASH = '098446ba6386797480a64ca51b2e102ce61ab8c9341a2c1351ac21f4c4243d11'
LICENSE_IV = '9a768b8203c057bcad755c7e0c5b888b'
FLAG_IV = '3f7a6de5b89d6ead39b6c2ebdf852ed0'


required_fields = [
    'first_name', 'last_name', 'dob', 'nickname', 
    'team_name', 'email', 'phone', 'card_number',
    'card_expiration', 'cvc', 'ctf', 'category', 
    'sponsor', 'os', 'oss_css', 'promo'
]


preferences = {
    "category" : {
        "choices" : ['Stego', 'OSINT'],
        "error_msg" : "Your category preference violates our Terms & Conditions.",
    },
    "os" : { 
        "choices" : ["Windows"],
        "error_msg" : "Your OS preference violates our Terms & Conditions.",
    },
     "oss_css" : {
        "choices" : ['CSS'],
        "error_msg" : "Preferring Open-Source Software violates our Terms & Conditions.",
    }   
}


app = Flask(__name__)
app.secret_key = os.urandom(64)


@app.route('/')
def serve_index():
    return render_template('index.html')


@app.route('/purchase', methods=['POST'])
def serve_purchase():

    fields = request.form.keys()

    for field in required_fields:
        if field not in fields:
            flash('Please submit all required information.')
            return render_template('result.html', error=True)

    for pref in preferences.keys():
        if request.form.get(pref) not in preferences[pref]['choices']:
            flash(preferences[pref]['error_msg'])
            flash("Please change your preference and try again.")
            return render_template('result.html', error=True)

    promo_code = request.form.get('promo')

    if not promo_code:
        flash("Your request will be processed in the next 72 hours.")
        flash("If you have a valid CoR Promo Code, it's time to use it!")
        flash("You could be the lucky winner of a free CoR Flag License!")
        return render_template('result.html', error=False)

    if sha256(promo_code) != PROMO_CODE_HASH:
        flash("You submitted an invalid CoR Promo Code.")
        flash("This is not fun.")
        return render_template('result.html', error=True)
    
    aes = AESDecryptor(promo_code.encode(), unhexlify(LICENSE_IV))
    cor_license = aes.decrypt(base64.b64decode(LICENSE))

    flash("Congratulations!")
    flash("You are the lucky winner of a free CoR Flag License!")
    flash(Markup(f"Please visit <a href='https://flag-license.cor.team/unlock?license={cor_license}'>https://flag-license.cor.team/unlock?license={cor_license}</a> to unlock the flag!"))    

    return render_template('result.html', error=False)


@app.route('/unlock', methods=['GET'])
def serve_flag():

    cor_license = request.args.get('license')

    if not cor_license or sha256(cor_license) != LICENSE_HASH:
        return redirect('/')

    aes = AESDecryptor(cor_license.encode(), unhexlify(FLAG_IV))
    flag = aes.decrypt(base64.b64decode(FLAG))
    flash(flag)

    return render_template('unlock.html', error=False)


@app.route('/flag')
def serve_pls():
    return f"corctf{{{hashlib.md5(os.urandom(32)).hexdigest()}}}"


if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000)
