import React, { useCallback, useEffect, useState } from 'react'
import './App.css'

interface Props {
  clientId: string
  g: string
  p: string
  setAuthenticated: (auth: boolean) => void
  x: string
  zkpLib: typeof import('pwd-dl-zkp-fe-lib')
}

export default function Authenticate({ clientId, g, p, setAuthenticated, x, zkpLib }: Props) {
  const [btnEnabled, setBtnEnabled] = useState(true)
  const [nOfTries, setNOfTries] = useState(0)
  const [nOfInvalid, setNOfInvalids] = useState(0)

  const authenticate = useCallback(() => {
    setBtnEnabled(false)

    const r = zkpLib.gen_r(p)

    const c = zkpLib.calc_c(r, g, p)

    const controller = new AbortController()
    const { signal } = controller

    async function pickChoice() {
      const res = await fetch('http://localhost:8000/pick-choice', {
        cache: 'no-cache',
        body: JSON.stringify({ c }),
        headers: {
          'content-type': 'application/json',
          'x-client-id': clientId,
        },
        method: 'POST',
        mode: 'cors',
        signal,
      })

      const { choice } = await res.json()

      return choice
    }

    async function verify(choice: string) {
      const res = await fetch('http://localhost:8000/verify', {
        cache: 'no-cache',
        body: JSON.stringify({ res: zkpLib.calc_choice(choice, x, r, p) }),
        headers: {
          'content-type': 'application/json',
          'x-client-id': clientId,
        },
        method: 'POST',
        mode: 'cors',
        signal,
      })

      const { cont, valid } = await res.json()

      setNOfTries(nOfTries + 1)
      setNOfInvalids(valid ? nOfInvalid : nOfInvalid + 1)

      return cont
    }

    async function verifyLoop(): Promise<void> {
      const choice = await pickChoice()
      const shouldContinue = await verify(choice)

      if (shouldContinue) {
        return verifyLoop()
      }
    }

    async function authenticated() {
      const res = await fetch('http://localhost:8000/authenticated', {
        cache: 'no-cache',
        headers: {
          'x-client-id': clientId,
        },
        mode: 'cors',
        signal,
      })

      const { auth } = await res.json()

      setAuthenticated(auth)
    }

    verifyLoop()
      .then(authenticated)
      .catch((err) => {
        console.error(err)
        controller.abort()
      })

    setBtnEnabled(true)
  }, [zkpLib, x, g, p, nOfTries, nOfInvalid])

  return (
    <div>
      <p className="App-BigInt">p: {p?.toString() ?? ''}</p>

      <p className="App-BigInt">g: {g?.toString() ?? ''}</p>

      <div>
        <button disabled={!btnEnabled} onClick={() => authenticate()}>
          Authenticate
        </button>
      </div>

      <p>
        Invalid attempts: {nOfInvalid} / {nOfTries}
      </p>
    </div>
  )
}
