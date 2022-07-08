---
colorlinks: true
---
# 4-wide strategy with longest expected combos

## Problem Definition

A state $s$ is combosed of 2 parts: $f \in F$ that represents the field and the hold piece, and $q \in Q$ that represents the sequence.
$$s = (f, q) \in F \times Q.$$

Given $f \in F$ and a piece $a \in A$, we can drop the piece at difference places and get different $f'$s.
We use function $\delta$ to denote this process.
$$\delta: F \times A \to \mathcal{P}(F).$$
Note that in this problem, we only discuss drops that cause a line clear.
So $\delta$ can map to $\varnothing$ on certain inputs.

Now consider the transition between states.

In a transition, the sequence $q \in Q$ is first updated according to the conditional distribution $P$ to give the new sequence $q' \in Q$ and a piece $a \in A$.
$$q', a | q \sim P.$$

Then the player should choose a $f' \in \delta(f, a)$, which is denoted as the policy $\pi$.
$$\pi: F \times A \times Q \rightharpoonup F.$$
If $\delta(f, a) = \varnothing$, $\pi(f, a, q')$ maps to nothing.

For a given policy $\pi$, we define its expected value $E_\pi$ on each state $s$ as follows.
$$
\begin{aligned}
  E_\pi(s)
    &= E_\pi(f, q),\\
    &= \mathbb{E}_{q', a|q \sim P} \left[\begin{cases}
        E_\pi(\pi(f, a, q'), q') + 1, & \delta(f, a) \neq \varnothing,\\
        0, &\delta(f, a) = \varnothing.
      \end{cases}\right]
\end{aligned}
$$

We want to get the best policy $\pi^*$ that maximize $E_\pi$:
$$
\pi^* = \arg \max_\pi E_\pi
$$

This is called [Markov decision process](https://en.wikipedia.org/wiki/Markov_decision_process), which has been extensively solved by previous researchers.

Because there is no actual difference between different $a$s if they give the same $\delta(f, a)$ for a given $f$s, the previous abstraction can be further reformed as follows to get rid of $a$.
$$
\begin{aligned}
  \mathcal{P}(F \times Q) | F \times Q &\sim P',\\
  P'(\delta(f, a) \times \{q'\}| (f, q)) &= \sum_{a': \delta(f, a') = \delta(f, a)} P(q', a'|q),\\
  P'(\varnothing | (f, q)) &= \sum_{q', a: \delta(f, a) = \varnothing} P(q', a|q).
\end{aligned}
$$

To ease our description of the algorithm, we will call any $S \subseteq F \times Q$ to be one continuation of $s$, if $P'(S|s) \neq 0$.

## Method

### Value Iteration
We solve this problem via [value iteration](https://en.wikipedia.org/wiki/Value_iteration) algorithm, a classical method to solve Markov decision process.
$$
\begin{aligned}
  V_{i+1}(f, q)
    &= \mathbb{E}_{q', a|q \sim P} \left[\begin{cases}
        \max_{f' \in \delta(f, a)} V_i(f', q') + 1, & \delta(f, a) \neq \varnothing,\\
        0, &\delta(f, a) = \varnothing,
      \end{cases}\right]\\
    &= \mathbb{E}_{q', a|q \sim P} \max \{0\} \cup \{V_i(f', q') + 1 | f' \in \delta(f, a)\}.\\
  V_0(\cdot) &= 0.
\end{aligned}
$$

The algirithm can be written as follows with $P'$.
$$V_{i+1}(s) = \mathbb{E}_{S|s \sim P'} \max\{0\} \cup \{V_i(s') + 1 | s' \in S\}.$$

### Minimization

We want to reduce the number of states.

If we calculate $V_i$ in the trivial way, the number of states can easily be too big to handle.
For example, with 3 residuals, no hold and no preview with random sequence pattern (usually called ``MPH''), $|F| = 40, |Q| = 1$.
But with one hold and 6 previews, $|F| = 40 \times 7 = 280$, and $|Q| = 7^6 = 117649$, so the total number of states is $280 \times 117649 = 32941720$.

And, there does exist redundant calculation that can be eliminated by merging states when calculating $V_i$.
For example, if we map to its symmetric counterpart both the field and every single piece in the hold and preview (i.e. `J, L, S, Z, I, O, T -> L, J, Z, S, I, O, T`), the $V_i$ should remain the same (ignoring the asymmetric I SRS rotation).

The minimization algorithm is based on [partition refinement](https://en.wikipedia.org/wiki/Partition_refinement), and very similar to [Hopcroft's DFA minimization algorithm](https://en.wikipedia.org/wiki/DFA_minimization#Hopcroft's_algorithm).

The algorithm will give a paritition of all states, so member states in the same group will always have the same $V$.

The basic starting point is that, if there is a one-to-one correspondence between the continuations of two states $s$ and $s'$, such that any pair of two corresponding continuations have the same probability and the same max $V_i$, then $V_{i+1}(s) = V_{i+1}(s')$.

The algorithm starts with all states in the same group.
For a given state partition, if all member states in a certain group do not always give the same $V$ assuming the partition is correct, this group will be split into different groups where each member in the same new group will give the same $V$ assuming the given partition is correct, and then the split group will be replaced by these new groups to form a new partition.
The algorithm stops when no group can be further split.

This algorithm terminates because groups are only split but not merged, and there is one partition that cannot be further refined, where each state forms its own group.
