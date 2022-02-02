import React, { useEffect, useState } from 'react'
import './App.css'
import Authenticate from './Authenticate'
import Password from './Password'

export default function App() {
  const [zkpLib, setZkpLib] = useState<typeof import('pwd-dl-zkp-fe-lib')>()
  const [x, setX] = useState<string>()
  const [clientId, setClientId] = useState<string>()
  const [p, setP] = useState<string>()
  const [g, setG] = useState<string>()
  const [authenticated, setAuthenticated] = useState(false)

  useEffect(() => {
    import('pwd-dl-zkp-fe-lib').then(async (lib) => {
      await lib.default()

      setZkpLib(lib)
    })
  }, [])

  return (
    <div className="App">
      <header className="App-header">
        <p>Set a password and verify it.</p>
      </header>

      <main className="App-main">
        {authenticated ? (
          <p>Authenticated!</p>
        ) : (
          <>
            {zkpLib ? (
              <>
                <Password setClientId={setClientId} setG={setG} setP={setP} setX={setX} x={x} zkpLib={zkpLib} />

                {clientId && g && p && x ? (
                  <Authenticate
                    clientId={clientId}
                    g={g}
                    p={p}
                    setAuthenticated={setAuthenticated}
                    x={x}
                    zkpLib={zkpLib}
                  />
                ) : null}
              </>
            ) : null}
          </>
        )}
      </main>
    </div>
  )
}
