import React, { useCallback, useState } from 'react'

function encodeStringToBigInt(s: string): string {
  return s
    .split('')
    .map((c) => c.charCodeAt(0).toString(16))
    .join('')
}

interface Props {
  setClientId: (clientId: string) => void
  setG: (g: string) => void
  setP: (p: string) => void
  setX: (x: string) => void
  x?: string
  zkpLib: typeof import('pwd-dl-zkp-fe-lib')
}

export default function Password({ setClientId, setG, setP, setX, x, zkpLib }: Props) {
  const [btnEnabled, setBtnEnabled] = useState(true)
  const [tempX, setTempX] = useState<string>('')

  const setPassword = useCallback(
    (pwd) => {
      if (!pwd || pwd === '') {
        return
      }

      setBtnEnabled(false)

      const x = encodeStringToBigInt(pwd)
      setX(x)

      const controller = new AbortController()
      const { signal } = controller

      async function handshake() {
        const res = await fetch('http://localhost:8000/handshake', {
          cache: 'no-cache',
          mode: 'cors',
          signal,
        })

        const { clientId, p, g } = await res.json()

        if (!clientId) {
          throw new Error('No client id returned.')
        }

        setClientId(clientId)
        setP(p)
        setG(g)

        return { clientId, g, p }
      }

      async function setPublicKey({ clientId, g, p }: { clientId: string; g: string; p: string }) {
        const y = zkpLib.public_key(x, g, p)

        await fetch('http://localhost:8000/public-key', {
          cache: 'no-cache',
          body: JSON.stringify({ y }),
          headers: {
            'content-type': 'application/json',
            'x-client-id': clientId,
          },
          method: 'POST',
          mode: 'cors',
          signal,
        })
      }

      handshake()
        .then(setPublicKey)
        .catch((err) => {
          console.error(err)
          controller.abort()
        })

      setBtnEnabled(true)
    },
    [zkpLib],
  )

  return (
    <div>
      <label>
        Password: <input onChange={(e) => setTempX(e.target.value)} value={tempX} />
      </label>

      <button disabled={!btnEnabled} onClick={() => setPassword(tempX)}>
        Set
      </button>

      <p>HEX: {x?.toString() ?? ''}</p>
    </div>
  )
}
