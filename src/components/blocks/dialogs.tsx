import { useId } from "react";

import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Label } from "@/components/ui/label";
import { RadioGroup, RadioGroupItem } from "@/components/ui/radio-group";
import { Textarea } from "@/components/ui/textarea";

export function RatingDialog() {
  const id = useId();

  const ratings = [
    { icon: "😡", label: "Angry", value: "1" },
    { icon: "🙁", label: "Sad", value: "2" },
    { icon: "🙂", label: "Neutral", value: "3" },
    { icon: "😁", label: "Happy", value: "4" },
    { icon: "🤩", label: "Laughing", value: "5" },
  ];

  return (
    <Dialog>
      <form>
        <DialogTrigger render={<Button variant="outline" />}>
          Rating
        </DialogTrigger>
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle className="text-xl">
              Help us improve our product for you
            </DialogTitle>
          </DialogHeader>
          <form className="flex flex-col gap-4 pt-4">
            <fieldset className="space-y-4">
              <legend className="font-medium text-foreground text-sm leading-none">
                How would you like to describe your experience with the app
                today?
              </legend>
              <RadioGroup className="flex gap-1.5" defaultValue="5">
                {ratings.map((rating) => (
                  <label
                    className="relative flex size-9 cursor-pointer flex-col items-center justify-center rounded-full border border-input text-center text-xl shadow-xs outline-none transition-[color,box-shadow] has-data-disabled:cursor-not-allowed has-data-[state=checked]:border-sky-600 has-focus-visible:border-sky-600 has-data-[state=checked]:bg-sky-600/10 has-data-disabled:opacity-50 has-focus-visible:ring-[3px] has-focus-visible:ring-sky-600/50 dark:has-data-[state=checked]:border-sky-400 dark:has-focus-visible:border-sky-400 dark:has-data-[state=checked]:bg-sky-400/10 dark:has-focus-visible:ring-sky-600/50"
                    htmlFor={`${id}-${rating.value}`}
                    key={`${id}-${rating.value}`}
                  >
                    <RadioGroupItem
                      className="sr-only after:absolute after:inset-0"
                      id={`${id}-${rating.value}`}
                      value={rating.value}
                    />
                    {rating.icon}
                  </label>
                ))}
              </RadioGroup>
            </fieldset>
            <div className="grid grow-1 gap-3">
              <Textarea
                id="message-2"
                placeholder="Type your message here."
                required
              />
              <p className="text-muted-foreground text-sm">
                500/500 characters left
              </p>
            </div>
            <div className="flex gap-3">
              <Checkbox id="terms" />
              <Label htmlFor="terms">
                I consent to Shadcn Studio contacting me based on my feedback
              </Label>
            </div>

            <DialogFooter className="sm:justify-end">
              <DialogClose render={<Button variant="outline" />}>
                Cancel
              </DialogClose>
              <Button type="submit">Submit</Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </form>
    </Dialog>
  );
}
