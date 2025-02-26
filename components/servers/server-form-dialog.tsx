import {
    Dialog,
    DialogContent,
    DialogHeader,
    DialogOverlay,
    DialogTitle,
    DialogTrigger,
} from "@/components/ui/dialog";
import React from "react";

export default function ServerFormDialog({
    formTitle,
    children,
    form,
}: {
    formTitle: string;
    children: React.ReactNode;
    form: React.ReactNode;
}) {
    return (
        <Dialog defaultOpen={false}>
            <DialogTrigger asChild>{children}</DialogTrigger>
            <DialogContent className="z-40 w-[1000px]">
                <DialogHeader>
                    <DialogTitle>{formTitle}</DialogTitle>
                </DialogHeader>
                {form}
            </DialogContent>
        </Dialog>
    );
}
