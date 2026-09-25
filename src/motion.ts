import type { Transition, Variants } from "motion/react";

export const motionTokens = {
  duration: {
    micro: 0.14,
    normal: 0.21,
    large: 0.28,
  },
  ease: [0.22, 1, 0.36, 1] as const,
};

export const transitions = {
  micro: { duration: motionTokens.duration.micro, ease: motionTokens.ease } satisfies Transition,
  normal: { duration: motionTokens.duration.normal, ease: motionTokens.ease } satisfies Transition,
  large: { duration: motionTokens.duration.large, ease: motionTokens.ease } satisfies Transition,
  layout: { type: "spring", stiffness: 420, damping: 38, mass: 0.8 } satisfies Transition,
};

export const viewVariants: Variants = {
  initial: (direction: number) => ({ opacity: 0, x: direction * 10 }),
  enter: { opacity: 1, x: 0, transition: transitions.normal },
  exit: (direction: number) => ({
    opacity: 0,
    x: direction * -6,
    transition: { duration: motionTokens.duration.micro, ease: motionTokens.ease },
  }),
};

export const panelVariants: Variants = {
  initial: (direction: number) => ({ opacity: 0, x: direction * 8 }),
  enter: { opacity: 1, x: 0, transition: transitions.normal },
  exit: (direction: number) => ({
    opacity: 0,
    x: direction * -5,
    transition: { duration: motionTokens.duration.micro, ease: motionTokens.ease },
  }),
};

export const collapseVariants: Variants = {
  initial: { opacity: 0, height: 0, y: -4 },
  enter: { opacity: 1, height: "auto", y: 0, transition: transitions.normal },
  exit: {
    opacity: 0,
    height: 0,
    y: -3,
    transition: { duration: motionTokens.duration.micro, ease: motionTokens.ease },
  },
};

export const listItemVariants: Variants = {
  initial: { opacity: 0, y: 5 },
  enter: { opacity: 1, y: 0, transition: transitions.normal },
  exit: { opacity: 0, x: -8, transition: transitions.micro },
};
