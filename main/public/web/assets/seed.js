/**
 * 默认题库 —— 3 个 Deck × 5 题
 */
export const SEED = {
  decks: [
    {
      id: 'deck_seed_neuro',
      title: '神经科学入门',
      subject: '生物学',
      desc: '神经递质、突触、髓鞘等基础概念，10 分钟搞定。',
      createdAt: Date.now() - 86400000 * 3,
      scheduledAt: Date.now() + 86400000,        // 明天
      lastPracticed: null,
      mastery: 0,
    },
    {
      id: 'deck_seed_vocab',
      title: '英语词汇 · 高频抽象名词',
      subject: '英语',
      desc: '中英对照词卡，适合"深度练习"模式逐词巩固。',
      createdAt: Date.now() - 86400000 * 2,
      scheduledAt: Date.now() + 86400000 * 3,    // 三天后
      lastPracticed: null,
      mastery: 0,
    },
    {
      id: 'deck_seed_physics',
      title: '物理常识 · 力学与热力学',
      subject: '物理',
      desc: '牛顿三定律、能量守恒、熵增定律等核心命题。',
      createdAt: Date.now() - 86400000,
      scheduledAt: Date.now() + 86400000 * 7,    // 一周后
      lastPracticed: null,
      mastery: 0,
    },
  ],
  cards: [
    // --- 神经科学 5 题 ---
    {
      id: 'card_n1', deckId: 'deck_seed_neuro',
      prompt: '哪种神经递质主要负责调节情绪、预防抑郁？',
      options: [
        { text: '多巴胺 Dopamine', correct: false },
        { text: '血清素 Serotonin', correct: true },
        { text: '乙酰胆碱 Acetylcholine', correct: false },
        { text: 'GABA 伽马氨基丁酸', correct: false },
      ],
      hint: '常被叫做"快乐荷尔蒙"，许多抗抑郁药通过提高它的浓度起效。',
    },
    {
      id: 'card_n2', deckId: 'deck_seed_neuro',
      prompt: '神经元中髓鞘 (myelin sheath) 的主要功能是？',
      options: [
        { text: '储存神经递质', correct: false },
        { text: '加快电信号传播速度', correct: true },
        { text: '为胞体提供结构支撑', correct: false },
        { text: '清除多余的离子', correct: false },
      ],
      hint: '多发性硬化症（MS）就是髓鞘受损导致的。',
    },
    {
      id: 'card_n3', deckId: 'deck_seed_neuro',
      prompt: '突触传递主要依靠哪种方式？',
      options: [
        { text: '电流直接跳跃', correct: false },
        { text: '化学神经递质释放与受体结合', correct: true },
        { text: '光信号', correct: false },
        { text: '机械振动', correct: false },
      ],
    },
    {
      id: 'card_n4', deckId: 'deck_seed_neuro',
      prompt: '海马体 (hippocampus) 的核心功能是？',
      options: [
        { text: '调节心率', correct: false },
        { text: '形成长期记忆', correct: true },
        { text: '控制平衡', correct: false },
        { text: '处理视觉', correct: false },
      ],
    },
    {
      id: 'card_n5', deckId: 'deck_seed_neuro',
      prompt: '动作电位 (action potential) 的"全有或全无"是什么意思？',
      options: [
        { text: '强度随刺激线性增加', correct: false },
        { text: '达到阈值后强度固定，否则不产生', correct: true },
        { text: '只在白天产生', correct: false },
        { text: '只在突触后膜产生', correct: false },
      ],
    },

    // --- 英语词汇 5 题（适配 deep 模式的"词卡式"展示）---
    {
      id: 'card_v1', deckId: 'deck_seed_vocab',
      prompt: '环境',
      lang: 'zh-en',
      options: [
        { text: 'Experience', correct: false },
        { text: 'Environment', correct: true },
        { text: 'Economy', correct: false },
      ],
      hint: 'env- 前缀来自 "envelop"，意为"包围"。',
    },
    {
      id: 'card_v2', deckId: 'deck_seed_vocab',
      prompt: '本质',
      lang: 'zh-en',
      options: [
        { text: 'Essence', correct: true },
        { text: 'Existence', correct: false },
        { text: 'Evidence', correct: false },
      ],
    },
    {
      id: 'card_v3', deckId: 'deck_seed_vocab',
      prompt: '同情',
      lang: 'zh-en',
      options: [
        { text: 'Empathy', correct: false },
        { text: 'Sympathy', correct: true },
        { text: 'Apathy', correct: false },
      ],
      hint: 'Empathy 是"共情"——感同身受；Sympathy 是"同情"——为对方感到难过。',
    },
    {
      id: 'card_v4', deckId: 'deck_seed_vocab',
      prompt: '矛盾',
      lang: 'zh-en',
      options: [
        { text: 'Conflict', correct: false },
        { text: 'Contradiction', correct: true },
        { text: 'Confusion', correct: false },
      ],
    },
    {
      id: 'card_v5', deckId: 'deck_seed_vocab',
      prompt: '韧性',
      lang: 'zh-en',
      options: [
        { text: 'Resilience', correct: true },
        { text: 'Reliance', correct: false },
        { text: 'Reluctance', correct: false },
      ],
    },

    // --- 物理 5 题 ---
    {
      id: 'card_p1', deckId: 'deck_seed_physics',
      prompt: '牛顿第二定律的数学表述是？',
      options: [
        { text: 'F = ma', correct: true },
        { text: 'E = mc²', correct: false },
        { text: 'F = G·m₁m₂/r²', correct: false },
        { text: 'PV = nRT', correct: false },
      ],
    },
    {
      id: 'card_p2', deckId: 'deck_seed_physics',
      prompt: '孤立系统的熵 (entropy)：',
      options: [
        { text: '总是减少', correct: false },
        { text: '总是不减（增大或不变）', correct: true },
        { text: '恒定不变', correct: false },
        { text: '随机变化', correct: false },
      ],
      hint: '这是热力学第二定律的核心。',
    },
    {
      id: 'card_p3', deckId: 'deck_seed_physics',
      prompt: '一辆静止的车，外力为零时它会？',
      options: [
        { text: '自动加速', correct: false },
        { text: '保持静止（惯性）', correct: true },
        { text: '自动后退', correct: false },
        { text: '消失', correct: false },
      ],
    },
    {
      id: 'card_p4', deckId: 'deck_seed_physics',
      prompt: '一个 2kg 的物体以 3m/s 运动，动能是？',
      options: [
        { text: '3 J', correct: false },
        { text: '6 J', correct: false },
        { text: '9 J', correct: true },
        { text: '18 J', correct: false },
      ],
      hint: 'Ek = ½ m v²',
    },
    {
      id: 'card_p5', deckId: 'deck_seed_physics',
      prompt: '光在真空中的传播速度约为？',
      options: [
        { text: '3 × 10⁵ m/s', correct: false },
        { text: '3 × 10⁸ m/s', correct: true },
        { text: '3 × 10¹⁰ m/s', correct: false },
        { text: '3 × 10⁶ m/s', correct: false },
      ],
    },
  ],
  sessions: [],
};
