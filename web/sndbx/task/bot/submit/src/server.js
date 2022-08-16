const listen = (opts, handler) => {
  const http = require('http')
  const getRawBody = require('raw-body')

  const port = process.env.PORT
  http.createServer(async (req, res) => {
    if (opts.subscribe && req.method !== 'POST') {
      res.writeHead(405).end()
      return
    }
    let reqBody
    if (req.method !== 'GET') {
      try {
        reqBody = await getRawBody(req, {
          length: req.headers['content-length'],
          limit: '20kb',
          encoding: 'utf8'
        })
      } catch {
        res.writeHead(413).end()
        return
      }
    }
    if (opts.subscribe) {
      const data = JSON.parse(reqBody).message.data
      await handler({
        message: JSON.parse(Buffer.from(data, 'base64').toString())
      })
      res.writeHead(204).end()
    } else {
      const idx = req.url.indexOf('?')
      let pathname
      let query
      if (idx === -1) {
        pathname = req.url
        query = {}
      } else {
        pathname = req.url.slice(0, idx)
        query = Object.fromEntries(new URLSearchParams(req.url.slice(idx)))
      }
      const { statusCode, headers, body } = await handler({
        pathname,
        query,
        method: req.method,
        headers: req.headers,
        body: reqBody
      })
      res.writeHead(statusCode, headers).end(body)
    }
  }).listen(port, () => {
    console.log('listening on', port)
  })
}

if (process.env.K_SERVICE) {
  exports.runtime = 'gcp'

  exports.run = listen

  if (process.env.APP_PUBSUB_TOPIC) {
    const { PubSub } = require('@google-cloud/pubsub')
    const topic = new PubSub().topic(process.env.APP_PUBSUB_TOPIC)
    exports.publish = message => topic.publishJSON(message)
  }
} else {
  exports.runtime = 'local'

  const http = require('http')

  exports.run = listen

  exports.publish = (message) => {
    const req = http.request('http://localhost:8000', { method: 'POST' })
    req.end(JSON.stringify({
      message: {
        data: Buffer.from(JSON.stringify(message)).toString('base64')
      }
    }))
  }
}
