import { cn } from "@/lib/utils";
import { useEffect, useState } from "react";

interface CountdownProps extends React.ComponentProps<"div"> {
    seconds: number;
    preText?: string;
}

export default function CountdownTimer({
    seconds,
    preText = "",
    className,
    ...props
}: CountdownProps) {
    const [counter, setCounter] = useState(0);
    useEffect(() => {
        const now = Math.floor(new Date().getTime() / 1000);

        setCounter(seconds - now);
    });

    useEffect(() => {
        let x: NodeJS.Timeout;
        if (counter > 0) {
            x = setTimeout(() => setCounter(counter - 1), 1000);
        }
    }, [counter]);

    function handleDisplay() {
        if (counter <= 0) {
            return `EXPIRED`;
        } else {
            const days = Math.floor(counter / (60 * 60 * 24));
            const hours = Math.floor((counter % (60 * 60 * 24)) / (60 * 60));
            const minutes = Math.floor((counter % (60 * 60)) / 60);
            const secs = Math.floor(counter % 60);
            let force = false;
            let display = ""
            if (days > 0){
                force = true;
                display += days + "d "
            }
            if (hours > 0 || force){
                force = true;
                display += hours + "h "
            }
            if (minutes > 0 || force){
                force = true;
                display += minutes + "m "
            }
            if (secs > 0 || force){
                force = true;
                display +=  secs + "s "
            }
            return display;
        }
    }

    return (
        <div className={cn(className)} {...props}>
            {preText}
            {handleDisplay()}
        </div>
    );
}
