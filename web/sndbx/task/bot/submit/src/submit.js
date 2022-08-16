const path = require('path')
const fs = require('fs')
const mustache = require('mustache')
const got = require('got')
const server = require('./server')

const submitPage = fs.readFileSync(path.join(__dirname, 'submit.html')).toString()

server.run({}, async (req) => {
  if (req.method === 'GET') {
    const page = mustache.render(submitPage, {
      recaptcha_site: process.env.APP_RECAPTCHA_SITE,
      msg: req.query.msg,
      js: req.query.js
    })
    return {
      statusCode: 200,
      headers: { 'content-type': 'text/html' },
      body: page
    }
  }
  if (req.method !== 'POST') {
    return { statusCode: 405 }
  }
  const body = new URLSearchParams(req.body)
  const send = msg => ({
    statusCode: 302,
    headers: {
      location: `?js=${encodeURIComponent(body.get('js'))}&msg=${encodeURIComponent(msg)}`
    }
  })
  if (process.env.APP_RECAPTCHA_SITE) {
    const recaptchaRes = await got({
      url: 'https://www.google.com/recaptcha/api/siteverify',
      method: 'POST',
      responseType: 'json',
      form: {
        secret: process.env.APP_RECAPTCHA_SECRET,
        response: body.get('recaptcha_code')
      }
    })
    if (!recaptchaRes.body.success) {
      return send('The reCAPTCHA is invalid.')
    }
  }
  const js = body.get('js')
  const payload = { js }
  console.log('publishing', JSON.stringify(payload))
  await server.publish(payload)
  return send('The admin will visit your URL.')
})
