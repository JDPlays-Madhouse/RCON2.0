"use client";

import { useState } from "react"

export default function SubCounter() {
    const [count, setCount] = useState(0)
    return <div onClick={() => { setCount(c => c += 1) }}>Subs: {count}</div>
}
