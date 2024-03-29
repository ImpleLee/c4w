# 期望连击长度最长的4w策略

## 问题介绍

在俄罗斯方块中，4-wide是一种制造长连击以获取高攻击的战术。在该战术中，预备工作为堆高6列而留下连续的4列不被堆高。由于每个俄罗斯方块都包含4个小块，将该块置入该4列并恰好造成一次消行时，放块增加的4小块和消行减少的4小块恰好抵消，导致该4列内的小块数目不变，进而可以期待下一块继续放入该4列而继续造成消行，从而达成长连击的目的。

对同一个地形和手头的方块，可能可以有多种不同的方块方式，它们可能都能造成消行，但由于留下了不同的地形，最终产生的连击数不一定相同。为了达到更长的连击数，我们希望找到一个较好的策略。

我们可以把4-wide理想化成场地为4行宽的俄罗斯方块（而不需要堆高其余6列）。于是，可以研究这样一个问题：在4行宽的场地中，给定场上的已知情况（即玩家一般能看到的所有信息），给出一个策略使得期望的连击长度最长。

### 背景

为了不熟悉我们的俄罗斯方块的设定的读者，我们在这里介绍一下我们考虑的俄罗斯方块的设定。

我们直接从游戏的中途开始考虑。
玩家可以操作一个系统给定的块，将其移动到某些系统允许的位置，并按下锁定键，这时填满的行从游戏中被移除，之后玩家拿到一个新的系统给定的块并重复该过程。

系统中有一个被称为暂存块的区域，其中保存着一个方块。玩家在操作系统给定块时，也可以通过hold操作，将手头的块与暂存块互换，使用暂存块来进行消行。此后暂存块就是玩家刚刚存入的块，直到玩家下一次通过hold操作来交换手头的块与暂存块为止。

玩家拿到新的系统给定的块的过程被称为出块。出块过程并不是随意出块的，而是根据之前的出块历史，来有概率地在若干种块中选择一种出。

我们的游戏目标是尽可能多地连续消行，即将系统给出的每个块（或者暂存块）用于消行。

## 问题定义

我们用状态 $s$ 表示从游戏中移除填满的块后一瞬间系统的状态。之所以使用这一时刻作为计算使用的状态是为了不记录当前块，减小状态空间的大小。

状态 $s \in S$ 由两部分组成： $f \in F$ 表示场地和hold， $q \in Q$ 表示序列。
$$s = (f, q) \in F \times Q = S.$$

下面考虑状态 $s$ 之间的转移。

序列 $q \in Q$ 首先按照条件概率分布 $P$ 产生新的序列 $q' \in Q$ 和一个块 $a \in A$ （即出块）。
$$q', a | q \sim P.$$

给定 $f \in F$ 和一个块 $a \in A$ ，可以把块落在不同位置（或者使用hold再落块）得到不同的 $f'$ 。
用函数 $\delta$ 来表记这个过程。
$$\delta: F \times A \to \mathcal{P}(F).$$
注意在该问题中，我们只讨论能消行的落块。
因此 $\delta$ 在特定输入上可能会映射到 $\varnothing$ 。

出块后，玩家从合法落块位置中选择一个落块位置 $f' \in \delta(f, a)$ ，该过程表记为策略 $\pi$ 。
$$\pi: F \times A \times Q \rightharpoonup F.$$
如果 $\delta(f, a)$ 为空集，则 $\pi(f, a, q')$ 没有合法的选择。

对给定策略 $\pi$ ，定义其在状态 $s$ 上的期望长度 $E_ \pi$ 如下。

$$
  E_ \pi(s)
    = E_ \pi(f, q)
    = \mathbb{E}_ {q', a|q \sim P} \left[\begin{cases}
        E_ \pi(\pi(f, a, q'), q') + 1, & \delta(f, a) \neq \varnothing,\\
        0, &\delta(f, a) = \varnothing.
      \end{cases}\right]
$$

我们想找到能最大化 $E_ \pi$ 的最优策略 $\pi^\ast$ 。

$$
\pi^\ast = \arg \max_ \pi E_ \pi.
$$

这个问题叫[Markov决策过程](https://en.wikipedia.org/wiki/Markov_decision_process)，是已经被基本解决了的数学问题。

对一个 $f$ , 如果两个 $a$ 给出相同的 $\delta(f, a)$ ，这两个 $a$ 其实没什么区别。因此之前的抽象可以继续按如下方式修改来去掉 $a$ 。

$$
\begin{aligned}
  S \subseteq F \times Q | s &\sim P',\\
  P'(\delta(f, a) \times \{q'\}| (f, q)) &= \sum_ {a': \delta(f, a') = \delta(f, a)} P(q', a'|q),\\
  P'(\varnothing | (f, q)) &= \sum_ {q', a: \delta(f, a) = \varnothing} P(q', a|q).
\end{aligned}
$$

为了描述方便，对于 $S \subseteq F \times Q$ ，如果 $P'(S|s) \neq 0$ ，我们下面会把 $S$ 叫做 $s$ 的一个后继。

## 方法

### Value Iteration
我们用[value iteration](https://en.wikipedia.org/wiki/Value_iteration)算法来解决这个问题，这是一个用来求解Markov决策过程的经典算法。

$$
\begin{aligned}
  V_ {i+1}(f, q)
    &= \mathbb{E}_ {q', a|q \sim P} \left[\begin{cases}
        \max_ {f' \in \delta(f, a)} V_ i(f', q') + 1, & \delta(f, a) \neq \varnothing,\\
        0, &\delta(f, a) = \varnothing,
      \end{cases}\right]\\
    &= \mathbb{E}_ {q', a|q \sim P} \left[ \max \left\lbrace 0 \right\rbrace \cup \left\lbrace V_ i(f', q') + 1 \vert f' \in \delta(f, a) \right\rbrace \right].\\
  V_ 0(\cdot) &= 0.
\end{aligned}
$$

用 $P'$ 来写的话就是这样。

$$
V_ {i+1}(s) = \mathbb{E}_ {S|s \sim P'} \left[ \max \left\lbrace  0 \right\rbrace \cup \left\lbrace V_ i(s') + 1 \vert s' \in S \right\rbrace \right].
$$

### 最小化

我们想要减少状态总数。

如果只是用朴素的方法计算 $V_ i$ 的话，状态数很快就会超出我们的处理能力。
比如，在种3、无hold、无next、随机出块（即MPH）的情况下， $|F| = 40, |Q| = 1$ 。
但有一个hold和6个next时， $|F| = 40 \times 7 = 280$ ， $|Q| = 7^6 = 117649$ ，这样状态数就是 $280 \times 117649 = 32941720$ 。

而且，其实在计算 $V_ i$ 时确实有冗余的计算可以通过状态合并来消除。
比如，如果把场地、hold和next中的每个块都左右翻转过来，（即`J, L, S, Z, I, O, T -> L, J, Z, S, I, O, T`），该状态的 $V_ i$ 值应当保持不变（忽略不对称的SRS I旋）。

定义最优策略下两个状态 $s$ 和 $s'$ 的期望值相同为这两个状态之间的一种关系 $s \sim s'$ 。

观察到：如果两个状态 $s_ 1$ 和 $s_ 2$ 之间存在 $P'(S_ 1 | s_ 1)$ 和 $P'(S_ 2 | s_ 2)$ 的联合分布 $P(S_ 1, S_ 2 | s_ 1, s_ 2)$ 使得 $P(S_ 1, S_ 2 | s_ 1, s_ 2)$ 非零时 $\forall t_ 1 \in S_ 1, \exists t_ 2 \in S_ 2, t_ 1 \sim t_ 2$ 且 $\forall t_ 2 \in S_ 2, \exists t_ 1 \in S_ 1, t_ 1 \sim t_ 2$ （即约束到 $S_ 1 \times S_ 2$ 上的 $\sim$ 既是左完全的也是右完全的） ，那么也有 $s_ 1 \sim s_ 2$ 。

该观察实际上给出了一个推理规则，即从一个 $\sim$ 成立的集合映射到另一个 $\sim$ 成立的集合的函数 $F: \mathcal P(S \times S) \to \mathcal P(S \times S)$ 。
显然在 $P_ 1 \subseteq P_ 2$ 时，我们有 $F(P_ 1) \subseteq F(P_ 2)$ ，也就是说 $F$ 保持集合的包含关系。
由[Knaster-Tarski定理](https://en.wikipedia.org/wiki/Knaster%E2%80%93Tarski_theorem)，存在一个最大不动点 $\nu F \subseteq \mathcal P(S \times S)$ 。
$\nu F \subseteq {\sim}$ 。

为了计算 $\nu F$ ，我们可以从一个初始集合 $P_ 0 = \mathcal P(S \times S)$ 开始，迭代地计算 $P_ {i+1} = F(P_ i)$ ，直到 $P_ i = P_ {i+1}$ 为止。我们记这个 $F$ 的不动点为 $P_ k$ 。

 $\nu F \subseteq P_ 0 = \mathcal P(S \times S)$ ，故 $\nu F = F(\nu F) \subseteq F(P_ 0) = P_ 1$ ，故 $\nu F \subseteq P_ i, \forall i$ ，于是 $\nu F \subseteq P_ k$ ；又由于 $P_ k$ 是 $F$ 的不动点，所以 $P_ k \subseteq \nu F$ ，故 $P_ k = \nu F$ 。

注意到 $F$ 的输出是自反且对称的，且 $F$ 保持传递性，所以 $F$ 的输入是等价关系时，输出也是等价关系。
又因为 $P_ 0$ 是等价关系，所以 $P_ i$ 均为等价关系。
所以 $\nu F$ 是一个等价关系，于是我们可以用它来合并状态。

$P_ 0 = \mathcal P(S \times S) \supseteq P_ 1$ ， $P_ 1 = F(P_ 0) \supseteq F(P_ 1) = P_ 2$ ，故 $P_ 0 \supseteq P_ 1 \supseteq P_ 2 \supseteq \cdots$ ，但 $P_ 0$ 是有限的，所以该算法在有限步内停止。
在不动点以外，该算法在每一步至少增加一个等价类，所以最多迭代 $|S|$ 次。

因为 $P_ i$ 是等价关系，所以我们可以用等价类来表示 $P_ i$ ，于是 $P_ i$ 可以在 $O(|S|)$ 的空间复杂度下表示。

具体到实际的算法，起初，所有状态都属于同一等价类。
随后，我们对每个状态进行遍历，对于每个状态，我们遍历它的后继，对于每个后继，我们将其中的状态替换为它们的等价类的代表。
这样，按照状态的新后继，我们可以将状态分到不同的等价类中。
当不再有状态可以分到新的等价类中时，我们就得到了一个新的等价关系。

这个最小化算法的运行逻辑基于[分割调整](https://en.wikipedia.org/wiki/Partition_refinement)，与[Hopcroft有限自动机最小化算法](https://en.wikipedia.org/wiki/DFA_minimization#Hopcroft's_algorithm)很像（几乎就是同一个算法）。

### 剪枝

基于value iteration的大小比较有数值精度的问题，我们期待找到更多不基于具体数值的比较方法。

定义最优策略下状态 $s$ 的期望值小于等于状态 $s'$ 的期望值为这两个状态的一种关系，记为 $s \lesssim s'$ 。

观察到：如果两个状态 $s_ 1$ 和 $s_ 2$ 之间存在 $P'(S_ 1 | s_ 1)$ 和 $P'(S_ 2 | s_ 2)$ 的联合分布 $P(S_ 1, S_ 2 | s_ 1, s_ 2)$ 使得 $P(S_ 1, S_ 2 | s_ 1, s_ 2)$ 非零时 $\forall t_ 1 \in S_ 1, \exists t_ 2 \in S_ 2, t_ 1 \lesssim t_ 2$ （即约束到 $S_ 1 \times S_ 2$ 上的 $\lesssim$ 是左完全的） ，那么也有 $s_ 1 \lesssim s_ 2$ 。

类似于上一节，这也给出了从一个 $\lesssim$ 成立的集合映射到另一个 $\lesssim$ 成立的集合的函数 $F: \mathcal P(S \times S) \to \mathcal P(S \times S)$ ，而且它也保持集合的包含关系。
由[Knaster-Tarski定理](https://en.wikipedia.org/wiki/Knaster%E2%80%93Tarski_theorem)，存在一个最大不动点 $\nu F \subseteq \mathcal P(S \times S)$ 。
$\nu F \subseteq {\lesssim}$ 。

我们同样可以从 $P_ 0 = \mathcal P(S \times S)$ 开始迭代计算 $P_ {i+1} = F(P_ i)$ ，直到 $P_ i = P_ {i+1}$ ，这样我们就得到了 $P_ k = \nu F$ 。
其正确性证明同上。

注意到 $F$ 的输出是自反的，且 $F$ 保持传递性。
又因为 $P_ 0$ 是自反且传递的，所以 $P_ i$ 均为自反传递的，所以 $\nu F$ 也是自反传递的。

得到这样的自反传递关系后，我们可以用它来把每个后继集合替换为 $\nu F$ 下的极大元的集合，达到剪枝的目的。

算法会停止的证明同上。
在不动点之外， 该算法每步至少删除一对状态间的关系，所以最多迭代 $|S|^2$ 次。

实际的算法类似于上一节，但我们不仅要验证由 $P_ i$ 导出的等价类是否会分裂，还要验证 $P_ i$ 的[transitive reduction](https://en.wikipedia.org/wiki/Transitive_reduction)的每条边是否会被移除。
（todo：实现该算法。）
