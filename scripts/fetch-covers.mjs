import { mkdir, writeFile } from 'node:fs/promises'
import { existsSync } from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const root = path.dirname(fileURLToPath(import.meta.url))
const outDir = path.join(root, '..', 'public', 'covers')

const MAL_IDS = [
  52991,
  47917,
  9253,
  33352,
  457,
  46102,
  22135,
  9756,
  32281,
  52701,
  12189,
  42310,
  5680,
  28851,
  4181,
  35839,
  50265,
  30
]

const sleep = (ms) => new Promise((r) => setTimeout(r, ms))

async function fetchJson(url, attempt = 0) {
  const res = await fetch(url, { headers: { 'User-Agent': 'kiroku-dev-asset-fetch' } })
  if (res.status === 429) {
    if (attempt >= 4) throw new Error(`429 too many requests for ${url}`)
    const wait = 2000 * (attempt + 1)
    console.log(`  429, waiting ${wait}ms then retrying ${url}`)
    await sleep(wait)
    return fetchJson(url, attempt + 1)
  }
  if (!res.ok) throw new Error(`HTTP ${res.status} for ${url}`)
  return res.json()
}

async function fetchBuffer(url, attempt = 0) {
  const res = await fetch(url, { headers: { 'User-Agent': 'kiroku-dev-asset-fetch' } })
  if (res.status === 429) {
    if (attempt >= 4) throw new Error(`429 too many requests for ${url}`)
    await sleep(2000 * (attempt + 1))
    return fetchBuffer(url, attempt + 1)
  }
  if (!res.ok) throw new Error(`HTTP ${res.status} for ${url}`)
  return Buffer.from(await res.arrayBuffer())
}

await mkdir(outDir, { recursive: true })

for (const id of MAL_IDS) {
  const target = path.join(outDir, `${id}.jpg`)
  if (existsSync(target)) {
    console.log(`skip ${id} (exists)`)
    continue
  }
  try {
    const json = await fetchJson(`https://api.jikan.moe/v4/anime/${id}`)
    const imageUrl = json?.data?.images?.jpg?.large_image_url
    if (!imageUrl) throw new Error('no large_image_url')
    const title = json?.data?.title ?? ''
    const buf = await fetchBuffer(imageUrl)
    if (buf.length < 5000) throw new Error(`suspiciously small image (${buf.length}b)`)
    await writeFile(target, buf)
    console.log(`ok ${id} ${title} -> ${buf.length}b`)
  } catch (err) {
    console.error(`FAIL ${id}: ${err.message}`)
  }
  await sleep(1100)
}
console.log('done')
