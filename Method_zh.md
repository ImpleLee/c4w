---
colorlinks: true
CJKmainfont: Noto Serif CJK SC
mathfont: Latin Modern Math
header-includes: |
  ```{=latex}
  \setmathfont[range=\varnothing]{Asana Math}
  \DeclareMathAlphabet{\mathcal}{OMS}{cmsy}{m}{n}
  \let\mathbb\relax
  \DeclareMathAlphabet{\mathbb}{U}{msb}{m}{n}
  ```
---
# 期望combo长度最长的4w策略

## 问题定义

状态$s$由两部分组成：$f \in F$表示场地和hold，$q \in Q$表示序列。
$$s = (f, q) \in F \times Q.$$

给定$f \in F$和一个块$a \in A$，可以把块落在不同位置（或者使用hold再落块）得到不同的$f'$。
用函数$\delta$来表记这个过程。
$$\delta: F \times A \to \mathcal{P}(F).$$
注意在该问题中，我们只讨论能消行的落块。
因此$\delta$在特定输入上可能会映射到$\varnothing$。

下面考虑状态之间的转移。

序列$q \in Q$首先按照条件概率分布$P$产生新的序列$q' \in Q$和一个块$a \in A$（即出块）。
$$q', a | q \sim P.$$

之后玩家从合法落块位置中选择一个落块位置$f' \in \delta(f, a)$，该过程表记为策略$\pi$。
$$\pi: F \times A \times Q \rightharpoonup F.$$
如果$\delta(f, a)$为空集，则$\pi(f, a, q')$没有合法的选择。

对给定策略$\pi$，定义其在状态$s$上的期望长度$E_\pi$如下。
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

我们想找到能最大化$E_\pi$的最优策略$\pi^*$。
$$
\pi^* = \arg \max_\pi E_\pi
$$

这个问题叫[Markov决策过程](https://en.wikipedia.org/wiki/Markov_decision_process)，是已经被基本解决了的数学问题。

对一个$f$, 如果两个$a$给出相同的$\delta(f, a)$，这两个$a$其实没什么区别。因此之前的抽象可以继续按如下方式修改来去掉$a$。
$$
\begin{aligned}
  \mathcal{P}(F \times Q) | F \times Q &\sim P',\\
  P'(\delta(f, a) \times \{q'\}| (f, q)) &= \sum_{a': \delta(f, a') = \delta(f, a)} P(q', a'|q),\\
  P'(\varnothing | (f, q)) &= \sum_{q', a: \delta(f, a) = \varnothing} P(q', a|q).
\end{aligned}
$$

为了描述方便，对于$S \subseteq F \times Q$，如果$P'(S|s) \neq 0$，我们下面会把$S$叫做$s$的一个后继。

## 方法

### Value Iteration
我们用[value iteration](https://en.wikipedia.org/wiki/Value_iteration)算法来解决这个问题，这是一个用来求解Markov决策过程的经典算法。
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

用$P'$来写的话就是这样。
$$V_{i+1}(s) = \mathbb{E}_{S|s \sim P'} \max\{0\} \cup \{V_i(s') + 1 | s' \in S\}.$$

### 最小化

我们想要减少状态总数。

如果只是用朴素的方法计算$V_i$的话，状态很快就会变得太多处理不了。
比如，在种3、无hold、无next、随机出块（即MPH）的情况下，$|F| = 40, |Q| = 1$。
但有一个hold和6个next时，$|F| = 40 \times 7 = 280$，$|Q| = 7^6 = 117649$，这样状态数就是$280 \times 117649 = 32941720$。

而且，其实在计算$V_i$时确实有冗余的计算可以通过状态合并来消除。
比如，如果把场地、hold和next中的每个块都左右翻转过来，（即`J, L, S, Z, I, O, T -> L, J, Z, S, I, O, T`），该状态的$V_i$值应当保持不变（忽略不对称的SRS I旋）。

我们的最小化算法基于[分割调整](https://en.wikipedia.org/wiki/Partition_refinement)，与[Hopcroft有限自动机最小化算法](https://en.wikipedia.org/wiki/DFA_minimization#Hopcroft's_algorithm)很像。

算法最终会给出状态的一个分割，分在同一组里的状态的$V$总是彼此相等。

基本出发点是这样的：如果两个状态$s$和$s'$的后续之间存在一个一一对应，使得对应的两个后续的概率和其中最大的$V_i$都相同，那$V_{i+1}(s) = V_{i+1}(s')$。

算法开始时，先把所有状态放在同一组里。
对一个分割，如果其中某个组里的所有状态在假设该分割成立时并不一定给出一样的$V$的话，我们就把这个组分成多个新的组，其中每个组里的状态在假设原分割成立的条件下都一定算出一样的$V$，这样替代掉原来的组构成一个新的分割。
这个算法运行到每个组都不能再分开为止。

该算法一定会终止，因为所有组都只会被分开而不会被合并；而这样的加细有一个终点是不能再加细的，就是每个状态单独分成一组。
