import type { PersonalDraft } from '../types/anime'

export interface SeedRecord {
  subjectId: number
  draft: PersonalDraft
  createdAt: string
  updatedAt: string
}

export const seedRecords: SeedRecord[] = [
  {
    subjectId: 52991,
    createdAt: '2025-05-02T14:20:00.000Z',
    updatedAt: '2025-06-14T09:35:00.000Z',
    draft: {
      score: 9.6,
      tier: 'S',
      status: 'completed',
      dimensions: { story: 5, characters: 5, direction: 5, animation: 4.5, music: 5 },
      review:
        '把"余生"两个字拍出了体温。芙莉莲每一次迟钝的回望，都像在替我问自己：和身边的人共度的那些平常日子，我有没有认真看过一眼。'
    }
  },
  {
    subjectId: 47917,
    createdAt: '2024-11-18T20:05:00.000Z',
    updatedAt: '2025-03-22T16:40:00.000Z',
    draft: {
      score: 8.8,
      tier: 'A',
      status: 'completed',
      dimensions: { story: 4, characters: 4.5, direction: 4.5, animation: 4.5, music: 5 },
      review:
        '把社恐的脑内小剧场拍成了 Live。livehouse 那几段演出的调度是真的顶，波奇酱迈出每一步我都跟着屏住呼吸。'
    }
  },
  {
    subjectId: 9253,
    createdAt: '2024-08-09T11:00:00.000Z',
    updatedAt: '2024-09-01T22:15:00.000Z',
    draft: {
      score: 9.2,
      tier: 'S',
      status: 'completed',
      dimensions: { story: 5, characters: 4.5, direction: 4.5, animation: 4, music: 4.5 },
      review:
        '前半程的日常铺垫几乎全是伏笔，回收的那一刻头皮发麻。冈部在无数条世界线里独自记得一切，这种孤独的重量少有作品能写出来。'
    }
  },
  {
    subjectId: 33352,
    createdAt: '2024-06-12T13:30:00.000Z',
    updatedAt: '2024-10-05T19:25:00.000Z',
    draft: {
      score: 8.4,
      tier: 'A',
      status: 'completed',
      dimensions: { story: 4, characters: 4, direction: 4.5, animation: 5, music: 4.5 },
      review:
        '画面精致到近乎奢侈。薇尔莉特学"爱"的过程有些单元剧略显刻意，但第十集那封跨越五十年的信还是把我击溃了。'
    }
  },
  {
    subjectId: 457,
    createdAt: '2024-03-15T10:00:00.000Z',
    updatedAt: '2024-07-20T15:10:00.000Z',
    draft: {
      score: 9.3,
      tier: 'S',
      status: 'completed',
      dimensions: { story: 4.5, characters: 4.5, direction: 5, animation: 4.5, music: 4.5 },
      review:
        '安静到能听见雪落的声音。银古从不评判，只是见证——这种克制的姿态本身就是对生命最大的敬意。失眠的夜里重看，比任何安慰都管用。'
    }
  },
  {
    subjectId: 46102,
    createdAt: '2025-01-25T21:40:00.000Z',
    updatedAt: '2025-02-10T12:30:00.000Z',
    draft: {
      score: 8.9,
      tier: 'A',
      status: 'completed',
      dimensions: { story: 5, characters: 4.5, direction: 4.5, animation: 4, music: 4 },
      review:
        '剧本密度高得惊人，每一句闲聊都是线索。动物皮套底下是近年最冷峻的都市群像，结局那一下反转让我倒回去重看了一整遍。'
    }
  },
  {
    subjectId: 22135,
    createdAt: '2024-05-30T18:00:00.000Z',
    updatedAt: '2024-08-16T20:45:00.000Z',
    draft: {
      score: 9.0,
      tier: 'A',
      status: 'completed',
      dimensions: { story: 4.5, characters: 4.5, direction: 5, animation: 4.5, music: 4.5 },
      review:
        '汤浅政明把乒乓球画成了存在主义。画风粗粝得像草稿，却比任何精致作画都更有生命力。"英雄参上"那一幕，看一次燃一次。'
    }
  },
  {
    subjectId: 9756,
    createdAt: '2024-04-22T14:15:00.000Z',
    updatedAt: '2024-05-01T23:00:00.000Z',
    draft: {
      score: 8.5,
      tier: 'B',
      status: 'completed',
      dimensions: { story: 4.5, characters: 4, direction: 4.5, animation: 4, music: 4.5 },
      review:
        '把魔法少女拍成了残酷的因果装置，开创性毋庸置疑。但坦白说，我对它的敬意多过热爱——看的时候更多在分析，而不是被打动。'
    }
  },
  {
    subjectId: 32281,
    createdAt: '2024-02-14T19:30:00.000Z',
    updatedAt: '2024-02-14T22:00:00.000Z',
    draft: {
      score: 7.9,
      tier: 'C',
      status: 'completed',
      dimensions: { story: 3.5, characters: 4, direction: 4, animation: 4.5, music: 4.5 },
      review:
        '画面和配乐无可挑剔，泷在山顶大喊的那场戏确实动人。但故事太顺了，顺到散场之后留不下多少余味——像一场做得极美的梦。'
    }
  },
  {
    subjectId: 52701,
    createdAt: '2025-04-10T12:00:00.000Z',
    updatedAt: '2025-06-01T17:20:00.000Z',
    draft: {
      score: null,
      tier: null,
      status: 'watching',
      dimensions: { story: null, characters: null, direction: null, animation: null, music: null },
      review: ''
    }
  },
  {
    subjectId: 12189,
    createdAt: '2024-09-08T15:20:00.000Z',
    updatedAt: '2024-11-12T10:35:00.000Z',
    draft: {
      score: 8.1,
      tier: 'B',
      status: 'completed',
      dimensions: { story: 4, characters: 4.5, direction: 4, animation: 4.5, music: 4 },
      review:
        '日常之谜的浪漫不在谜底，在于"有人愿意认真陪你把小事想清楚"。折木和千反田的距离感拿捏得极好，最后两集的分寸堪称教科书。'
    }
  },
  {
    subjectId: 42310,
    createdAt: '2025-03-05T20:50:00.000Z',
    updatedAt: '2025-03-18T21:15:00.000Z',
    draft: {
      score: 8.3,
      tier: 'B',
      status: 'completed',
      dimensions: { story: 4, characters: 4, direction: 4.5, animation: 4.5, music: 4.5 },
      review:
        '十集讲完一个干净利落的坠落故事。夜之城从不为谁停留，大卫燃尽自己的方式既是反抗也是认命，后劲比想象中大得多。'
    }
  }
]
