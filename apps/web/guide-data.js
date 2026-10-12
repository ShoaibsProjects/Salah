// Original educational summaries. Qur’an quotations are verified separately.
// Further-reading page numbers refer to the 2021 prayer guide; no PDF or publisher illustrations are bundled.
export const BOOK = "https://abukhadeejah.com/wp-content/uploads/2018/05/Simple-Guide-to-Prayer-2021-Website-2.0.pdf";
export const AUTHOR = "https://abukhadeejah.com/prophets-prayer-beginners-illustrations/";
export const prayers = [
  {
    "id": "fajr",
    "name": "Fajr",
    "arabic": "الفجر",
    "units": 2,
    "time": "Dawn",
    "voice": "At dawn, Fajr begins the day with worship and remembrance of Allah. Its two rakʿahs offer a quiet beginning before the day gathers pace.",
    "reading": "Individual prayer: recitation may be audible in both rakʿahs, without disturbing others."
  },
  {
    "id": "dhuhr",
    "name": "Dhuhr",
    "arabic": "الظهر",
    "units": 4,
    "time": "After midday",
    "voice": "Dhuhr brings a pause for worship after midday. Its four rakʿahs make room for remembrance amid work, responsibilities and ordinary life.",
    "reading": "The recitation is quiet in all four rakʿahs, with movement of the lips."
  },
  {
    "id": "asr",
    "name": "Asr",
    "arabic": "العصر",
    "units": 4,
    "time": "Afternoon",
    "voice": "Asr is the afternoon prayer, with four rakʿahs. As the day moves on, prayer renews attention to Allah and the responsibility of worship.",
    "reading": "The recitation is quiet in all four rakʿahs, with movement of the lips."
  },
  {
    "id": "maghrib",
    "name": "Maghrib",
    "arabic": "المغرب",
    "units": 3,
    "time": "After sunset",
    "voice": "Maghrib begins after sunset and has three rakʿahs. The passing of daylight marks another time to worship Allah and give thanks.",
    "reading": "Individual prayer: recitation may be audible in the first two rakʿahs; the third is quiet."
  },
  {
    "id": "isha",
    "name": "Isha",
    "arabic": "العشاء",
    "units": 4,
    "time": "Night",
    "voice": "Isha is the night prayer, with four rakʿahs. It brings worship into the close of the day, with praise, supplication and remembrance.",
    "reading": "Individual prayer: recitation may be audible in the first two rakʿahs; the last two are quiet."
  }
];
export const wudu = [
  [
    "Begin with the hands",
    "The hands are washed first. Each washed area may be washed once, twice or three times; washing is not repeated beyond three.",
    "Hands"
  ],
  [
    "Mouth & nose",
    "The mouth is rinsed, and water is gently drawn into the nose and expelled. Care is taken to avoid discomfort.",
    "Mouth & nose"
  ],
  [
    "Wash the face",
    "The whole face is washed, with attention to its edges so that water reaches the area to be washed.",
    "Face"
  ],
  [
    "Wash both arms",
    "The right hand and arm are washed, including the elbow, followed by the left. The elbows are included in the washing.",
    "Arms"
  ],
  [
    "Wipe the head",
    "With wet hands, the head is wiped from front to back and back to front. This is one wiping movement, rather than three washes.",
    "Head"
  ],
  [
    "Wipe the ears",
    "The inside of the ears is wiped with the index fingers, and the back of the ears with the thumbs.",
    "Ears"
  ],
  [
    "Wash both feet",
    "The right foot is washed, including the ankle, followed by the left. Care is taken around the heels and between the toes.",
    "Feet"
  ]
];
export const topics = [
  {
    "id": "begin",
    "category": "Heart & purpose",
    "title": "A place to begin",
    "pages": "3–4, 50–55",
    "body": [
      "Salah is worship offered to Allah alone, following the example of the Prophet Muhammad ﷺ. The five daily prayers give the day a rhythm of remembrance: Fajr has two rakʿahs, Dhuhr four, Asr four, Maghrib three and Isha four.",
      "Learning can begin with a small, steady step. Understanding the movements and words takes practice; a patient teacher can help a beginner grow in confidence."
    ],
    "links": [
      [
        "Sahih al-Bukhari 631 · following the Prophet’s prayer",
        "https://sunnah.com/bukhari:631"
      ]
    ],
    "related": [
      [
        "Explore a prayer",
        "#learn"
      ]
    ]
  },
  {
    "id": "conditions",
    "category": "Preparation",
    "title": "Preparing for prayer",
    "pages": "6–8",
    "body": [
      "Preparation includes the prayer’s time, purification, suitable covering, a clean body, clothes and place, and the direction of the Qiblah. The intention is held sincerely in the heart for Allah.",
      "The beginner path here uses loose, opaque clothing: men cover between the navel and knees and cover their shoulders; women cover the body except the face and hands. Personal questions about covering or purification are best discussed with a qualified teacher."
    ],
    "links": [],
    "related": [
      [
        "Preparation & wuḍū",
        "#prepare"
      ]
    ]
  },
  {
    "id": "purification",
    "category": "Preparation",
    "title": "Wuḍū, with care",
    "pages": "8–11",
    "body": [
      "Wuḍū prepares the body for prayer. The sequence covers the hands, mouth and nose, face, arms including elbows, head and ears, and feet including ankles. The head and ears are wiped; the other areas are washed.",
      "Careful washing need not become repeated checking. One, two or three washes are reported; the head is wiped once. When ghusl is required, wuḍū alone does not replace it."
    ],
    "links": [
      [
        "Sahih Muslim 235a · the description of wuḍū",
        "https://sunnah.com/muslim:235a"
      ]
    ],
    "related": [
      [
        "Follow the wuḍū steps",
        "#prepare"
      ]
    ]
  },
  {
    "id": "footwear",
    "category": "Preparation",
    "title": "Wiping over socks & footwear",
    "pages": "11–14",
    "body": [
      "Wiping over qualifying footwear is a concession during a later wuḍū. The feet are first washed during complete purification before the footwear is put on.",
      "The reported period is one day and night for a resident and three days and nights for a traveller. The start of that period, suitable footwear and questions about removal need careful guidance from a qualified teacher."
    ],
    "links": [
      [
        "Sahih Muslim 276a · the wiping periods",
        "https://sunnah.com/muslim:276a"
      ]
    ],
    "related": []
  },
  {
    "id": "tayammum",
    "category": "Preparation",
    "title": "When water is unavailable",
    "pages": "14",
    "body": [
      "Tayammum is purification with clean earth when water is unavailable. In the reported method, the palms touch the earth, excess dust is blown off, and the face and hands are wiped.",
      "Illness or inability to use water may also require guidance about tayammum. A qualified teacher can explain how the concession applies to a particular situation."
    ],
    "links": [
      [
        "Sahih al-Bukhari 343 · tayammum",
        "https://sunnah.com/bukhari:343"
      ]
    ],
    "related": []
  },
  {
    "id": "direction",
    "category": "Preparation",
    "title": "Qiblah & a clean prayer space",
    "pages": "17–20, 50–51",
    "body": [
      "The worshipper faces the Kaʿbah in Makkah as able. A reliable established direction is used; this app does not provide a reviewed Qiblah compass.",
      "A suitable clean place is enough for prayer. Mosque arrangements are respected, and places such as toilet areas and graveyards are avoided. A trusted teacher can help with uncertainty about purity or direction."
    ],
    "links": [],
    "related": []
  },
  {
    "id": "sutrah",
    "category": "Preparation",
    "title": "A boundary in front of the prayer",
    "pages": "20–22",
    "body": [
      "A sutrah is an upright boundary placed beyond the area of prostration. A wall, pillar or another suitable raised object can serve this purpose.",
      "Respect for another person’s prayer includes passing beyond their boundary, rather than walking between it and the worshipper. Questions about a particular incident can be taken to a qualified teacher."
    ],
    "links": [],
    "related": []
  },
  {
    "id": "intention",
    "category": "Heart & purpose",
    "title": "Sincerity & intention",
    "pages": "22–23",
    "body": [
      "Prayer begins with an intention directed to Allah. The worshipper knows which prayer is being offered; the intention belongs in the heart and does not need a spoken formula.",
      "Sahih al-Bukhari opens with the narration on intention and the reward of deeds. Learning the outward movements is joined to care for the purpose of the worship itself."
    ],
    "links": [
      [
        "Sahih al-Bukhari 1 · intention",
        "https://sunnah.com/bukhari:1"
      ]
    ],
    "related": []
  },
  {
    "id": "standing",
    "category": "Prayer",
    "title": "Standing before Allah",
    "pages": "23–26",
    "body": [
      "The obligatory prayer is offered standing when the worshipper is able. At the opening takbīr, the hands are raised, then the right hand is placed over the left at the chest. The gaze is directed toward the place of prostration.",
      "An opening supplication, seeking refuge with Allah, and recitation follow. When attention wanders, returning gently to the recitation helps restore the focus of the prayer."
    ],
    "links": [],
    "related": [
      [
        "The opening words",
        "#words/opening"
      ]
    ]
  },
  {
    "id": "fatihah",
    "category": "Prayer",
    "title": "Al-Fātiḥah: the Opening",
    "pages": "26–30",
    "body": [
      "Al-Fātiḥah is recited in each rakʿah of the individual prayer. Another known passage of Qur’an follows in the first two rakʿahs; the later rakʿahs use Al-Fātiḥah.",
      "Sahih Muslim 395a records Allah’s response to the servant’s recitation of Al-Fātiḥah. The report gives this familiar recitation a place of particular care and attention.",
      "Recitation behind an imam is learned with a trusted imam or teacher. The walkthrough here teaches an individual prayer."
    ],
    "links": [
      [
        "Sahih Muslim 395a · Al-Fātiḥah in prayer",
        "https://sunnah.com/muslim:395a"
      ]
    ],
    "related": [
      [
        "Read Al-Fātiḥah",
        "#words/fatihah"
      ]
    ]
  },
  {
    "id": "new-muslim",
    "category": "Heart & purpose",
    "title": "Learning the Arabic, gradually",
    "pages": "28–29",
    "body": [
      "Al-Fātiḥah can be learned a little at a time. Reading aids help with a beginning, but a teacher who listens can correct sounds that English letters cannot fully represent.",
      "A temporary remembrance is reported for someone who is genuinely unable to recite it while learning. A qualified teacher can explain this concession, while learning Al-Fātiḥah continues."
    ],
    "links": [
      [
        "Sunan Abi Dawud 832 · remembrance while learning",
        "https://sunnah.com/abudawud:832"
      ]
    ],
    "related": [
      [
        "Words while learning",
        "#words/beginner"
      ]
    ]
  },
  {
    "id": "bowing",
    "category": "Prayer",
    "title": "Bowing, then standing fully",
    "pages": "30–33",
    "body": [
      "In rukūʿ, the worshipper bows with the hands on the knees, keeping the back and head level as able. Allah is glorified, and the posture is allowed to settle. Qur’an is not recited in bowing.",
      "The worshipper then rises fully upright and praises Allah before moving to prostration. Sahih al-Bukhari 757 emphasizes settling in each posture rather than hurrying through the prayer."
    ],
    "links": [
      [
        "Sahih al-Bukhari 757 · calmness in the movements",
        "https://sunnah.com/bukhari:757"
      ]
    ],
    "related": [
      [
        "Words in bowing",
        "#words/ruku"
      ]
    ]
  },
  {
    "id": "prostration",
    "category": "Prayer",
    "title": "Sujūd: humility & supplication",
    "pages": "33–38",
    "body": [
      "In prostration, the forehead and nose, hands, knees and toes meet the ground as able. The forearms are lifted and the posture is allowed to settle.",
      "The Prophet ﷺ taught that a servant is nearest to his Lord in prostration and encouraged supplication there. Sujūd joins humility of the body with praise, need and hope in Allah.",
      "Between the two prostrations, the worshipper sits calmly and asks for forgiveness. Care is taken not to crowd or harm others."
    ],
    "links": [
      [
        "Sahih Muslim 482 · nearness in prostration",
        "https://sunnah.com/muslim:482"
      ],
      [
        "Sahih al-Bukhari 757 · settling in each posture",
        "https://sunnah.com/bukhari:757"
      ]
    ],
    "related": [
      [
        "Words in prostration",
        "#words/sujud"
      ],
      [
        "Between the prostrations",
        "#words/forgive"
      ]
    ]
  },
  {
    "id": "sittings",
    "category": "Prayer",
    "title": "The sittings & later rakʿahs",
    "pages": "38–46",
    "body": [
      "After the second rakʿah, the worshipper sits for tashahhud. In a three- or four-rakʿah prayer, the prayer then continues with the remaining rakʿahs.",
      "The final sitting includes the tashahhud, blessings upon the Prophet ﷺ, and supplication before the salām. Posture is learned with care and adapted to actual ability, without restricting a neighbour."
    ],
    "links": [],
    "related": []
  },
  {
    "id": "tashahhud",
    "category": "Prayer",
    "title": "The words of the tashahhud",
    "pages": "41, 45",
    "body": [
      "The tashahhud contains words of honour and worship for Allah, peace upon the Prophet ﷺ and Allah’s righteous servants, and the testimony of faith.",
      "The reported wording shown in the Words section can be practised slowly. Listening to a teacher helps with pronunciation and with learning the sitting itself."
    ],
    "links": [
      [
        "Sahih al-Bukhari 6265 · the reported tashahhud",
        "https://sunnah.com/bukhari:6265"
      ]
    ],
    "related": [
      [
        "Read the tashahhud",
        "#words/tashahhud"
      ]
    ]
  },
  {
    "id": "finish",
    "category": "Prayer",
    "title": "Supplication & closing with peace",
    "pages": "45–49",
    "body": [
      "In the final sitting, the worshipper seeks Allah’s protection from the four trials named in the reported supplication. Further authentic supplications may follow.",
      "The prayer closes with the salutation to the right and then the left. Remembrance continues after the prayer, beginning with seeking forgiveness and the words of peace reported in Sahih Muslim 591."
    ],
    "links": [
      [
        "Sahih Muslim 588a · seeking protection",
        "https://sunnah.com/muslim:588a"
      ],
      [
        "Sahih Muslim 591 · remembrance after prayer",
        "https://sunnah.com/muslim:591"
      ]
    ],
    "related": [
      [
        "Protection before the salām",
        "#words/protection"
      ],
      [
        "After the prayer",
        "#library/after"
      ]
    ]
  },
  {
    "id": "women",
    "category": "Everyday life",
    "title": "Learning as a woman",
    "pages": "7, 48–49",
    "body": [
      "The beginner sequence shown here teaches the prayer movements for women and men. Suitable covering and individual circumstances remain part of preparation.",
      "Menstruation and postnatal bleeding require specific religious guidance. A calendar or symptom alone is not used by this companion to decide whether prayer should be performed or made up."
    ],
    "links": [],
    "related": []
  },
  {
    "id": "ability",
    "category": "Everyday life",
    "title": "Prayer according to ability",
    "pages": "17–19",
    "body": [
      "Prayer remains an act of worship within the person’s ability. The Prophet ﷺ instructed prayer standing, sitting when standing is not possible, and on the side when sitting is not possible.",
      "Pain, illness or disability may require adapted movement. A qualified teacher can explain the religious guidance, while a clinician can advise about movements that affect health."
    ],
    "links": [
      [
        "Sahih al-Bukhari 1117 · prayer according to ability",
        "https://sunnah.com/bukhari:1117"
      ]
    ],
    "related": [
      [
        "Explore the prayer sequence",
        "#learn"
      ]
    ]
  },
  {
    "id": "travel",
    "category": "Everyday life",
    "title": "Prayer while travelling",
    "pages": "17–19",
    "body": [
      "Travel raises practical questions about shortening, combining, timing and a safe place to pray. The walkthrough here shows the ordinary resident prayer counts.",
      "The appropriate guidance depends on the journey and circumstances. Those details are best discussed with a qualified teacher before travel when possible."
    ],
    "links": [],
    "related": []
  },
  {
    "id": "times",
    "category": "Everyday life",
    "title": "Understanding prayer times",
    "pages": "4–6",
    "body": [
      "Prayer beginnings are connected to the signs of dawn, the sun passing its highest point, afternoon shadow, sunset and nightfall. Calculations estimate those beginnings; a mosque’s iqāmah is the time its congregation begins.",
      "This app’s calculation page remains a research preview. Local mosque guidance can help with choosing settings and checking a timetable. Personal rulings about missed prayers or end times need qualified guidance."
    ],
    "links": [
      [
        "Sahih Muslim 612a · the prayer times",
        "https://sunnah.com/muslim:612a"
      ]
    ],
    "related": []
  },
  {
    "id": "congregation",
    "category": "Everyday life",
    "title": "Prayer in congregation",
    "pages": "23, 29–30, 50",
    "body": [
      "Congregational prayer brings worshippers together in a shared act of worship. The Prophet ﷺ described its reward as twenty-seven times greater than prayer offered alone.",
      "The imam is followed: bowing and prostration take place after the imam’s movement, rather than ahead of it. The rows are straightened with consideration for those beside them.",
      "An unfamiliar mosque or an uncertain first visit can feel daunting. A quiet question to the imam can help with recitation, late arrival or a personal circumstance. Learning the details takes time."
    ],
    "links": [
      [
        "Sahih al-Bukhari 645 · the virtue of congregation",
        "https://sunnah.com/bukhari:645"
      ],
      [
        "Sahih al-Bukhari 722 · following the imam",
        "https://sunnah.com/bukhari:722"
      ]
    ],
    "related": [
      [
        "Learn the prayer sequence",
        "#learn"
      ]
    ]
  },
  {
    "id": "importance",
    "category": "Heart & purpose",
    "title": "Why Salah matters",
    "pages": "50–55",
    "body": [
      "Salah gathers praise, recitation, humility and supplication into worship of Allah. The five daily prayers bring remembrance into ordinary life, from dawn until night.",
      "Authentic reports describe forgiveness through the five prayers, with the qualification that major sins are avoided. Prostration is also described as a time of nearness to Allah and an occasion for supplication.",
      "A difficult day or a wandering mind calls for patient care, rather than harsh self-judgment. Steady learning, sincere repentance and help from a trusted teacher can support a return to prayer."
    ],
    "links": [
      [
        "Sahih Muslim 233a · prayer and forgiveness",
        "https://sunnah.com/muslim:233a"
      ],
      [
        "Sahih Muslim 482 · nearness in prostration",
        "https://sunnah.com/muslim:482"
      ]
    ],
    "related": [
      [
        "Begin with preparation",
        "#prepare"
      ]
    ],
    "verses": [
      "13:28",
      "29:45"
    ]
  },
  {
    "id": "beyond",
    "category": "Everyday life",
    "title": "When a question needs more care",
    "pages": "3–55 · scope note",
    "body": [
      "Forgetfulness, missed prayers, Jumuʿah, Eid, funeral prayer, travel, ghusl, menstruation and postnatal bleeding may need more guidance than a short learning screen provides.",
      "A qualified teacher can understand what happened and explain the next step. Asking for help is part of learning, and a careful answer begins with the person’s actual circumstances."
    ],
    "links": [],
    "related": []
  },
  {
    "id": "after",
    "category": "Prayer",
    "title": "Remembrance after the prayer",
    "body": [
      "After completing the prayer, the Prophet ﷺ sought forgiveness three times, then recited the supplication beginning Allāhumma antas-salām. These words keep remembrance close as ordinary life resumes.",
      "Sahih Muslim also records saying Subḥānallāh, Alḥamdu lillāh and Allāhu akbar thirty-three times each, then completing a hundred with the declaration of Allah’s oneness shown below. The number and wording belong to this reported practice."
    ],
    "links": [
      [
        "Sahih Muslim 591 · forgiveness & peace",
        "https://sunnah.com/muslim:591"
      ],
      [
        "Sahih Muslim 597a · remembrance after prayer",
        "https://sunnah.com/muslim:597a"
      ]
    ],
    "related": [
      [
        "Forgiveness & peace",
        "#words/after-forgive"
      ],
      [
        "The declaration of oneness",
        "#words/after-tahlil"
      ]
    ]
  }
];
export const words = [
  {
    "id": "takbir",
    "title": "The opening & movement takbīr",
    "arabic": "اللَّهُ أَكْبَرُ",
    "translit": "Allāhu akbar.",
    "meaning": "Allah is greater than all.",
    "pages": "23, 30, 33"
  },
  {
    "id": "opening",
    "title": "An opening supplication",
    "arabic": "سُبْحَانَكَ اللَّهُمَّ وَبِحَمْدِكَ وَتَبَارَكَ اسْمُكَ وَتَعَالَى جَدُّكَ وَلَا إِلَهَ غَيْرُكَ",
    "translit": "Subḥānaka Allāhumma wa biḥamdika, wa tabārakasmuka, wa taʿālā jadduka, wa lā ilāha ghayruk.",
    "meaning": "O Allah, You are free of imperfection, and praise belongs to You. Blessed is Your name and exalted is Your majesty. None is worthy of worship but You.",
    "pages": "26",
    "note": "One reported opening supplication, recited at the beginning of prayer."
  },
  {
    "id": "refuge",
    "title": "Seeking refuge before recitation",
    "arabic": "أَعُوذُ بِاللَّهِ مِنَ الشَّيْطَانِ الرَّجِيمِ مِنْ هَمْزِهِ وَنَفْخِهِ وَنَفْثِهِ",
    "translit": "Aʿūdhu billāhi minash-shayṭānir-rajīm, min hamzihi wa nafkhihi wa nafthih.",
    "meaning": "I seek Allah’s protection from the rejected Satan, from his incitement, pride and harmful inspiration.",
    "pages": "26–27"
  },
  {
    "id": "ruku",
    "title": "In bowing · rukūʿ",
    "arabic": "سُبْحَانَ رَبِّيَ الْعَظِيمِ",
    "translit": "Subḥāna rabbiyal-ʿaẓīm.",
    "meaning": "My Lord, the Magnificent, is free of imperfection.",
    "pages": "31–32",
    "note": "Allah is glorified in bowing while the posture remains settled."
  },
  {
    "id": "rise",
    "title": "Rising from bowing",
    "arabic": "سَمِعَ اللَّهُ لِمَنْ حَمِدَهُ",
    "translit": "Samiʿallāhu liman ḥamidah.",
    "meaning": "Allah hears the one who praises Him.",
    "pages": "32",
    "note": "For an individual prayer. Congregational practice is learned with an imam or teacher."
  },
  {
    "id": "praise",
    "title": "Once upright",
    "arabic": "رَبَّنَا وَلَكَ الْحَمْدُ",
    "translit": "Rabbanā wa lakal-ḥamd.",
    "meaning": "Our Lord, all praise belongs to You.",
    "pages": "33"
  },
  {
    "id": "sujud",
    "title": "In prostration · sujūd",
    "arabic": "سُبْحَانَ رَبِّيَ الْأَعْلَى",
    "translit": "Subḥāna rabbiyal-aʿlā.",
    "meaning": "My Lord, the Most High, is free of imperfection.",
    "pages": "35",
    "note": "Allah is glorified in prostration; supplication is encouraged here."
  },
  {
    "id": "forgive",
    "title": "Between the prostrations",
    "arabic": "رَبِّ اغْفِرْ لِي رَبِّ اغْفِرْ لِي",
    "translit": "Rabbighfir lī, rabbighfir lī.",
    "meaning": "My Lord, forgive me. My Lord, forgive me.",
    "pages": "37",
    "note": "A short supplication between the two prostrations."
  },
  {
    "id": "tashahhud",
    "title": "The tashahhud",
    "arabic": "التَّحِيَّاتُ لِلَّهِ وَالصَّلَوَاتُ وَالطَّيِّبَاتُ، السَّلَامُ عَلَى النَّبِيِّ وَرَحْمَةُ اللَّهِ وَبَرَكَاتُهُ، السَّلَامُ عَلَيْنَا وَعَلَى عِبَادِ اللَّهِ الصَّالِحِينَ، أَشْهَدُ أَنْ لَا إِلَهَ إِلَّا اللَّهُ، وَأَشْهَدُ أَنَّ مُحَمَّدًا عَبْدُهُ وَرَسُولُهُ",
    "translit": "At-taḥiyyātu lillāhi waṣ-ṣalawātu waṭ-ṭayyibāt. As-salāmu ʿalan-nabiyyi wa raḥmatullāhi wa barakātuh. As-salāmu ʿalaynā wa ʿalā ʿibādillāhiṣ-ṣāliḥīn. Ashhadu an lā ilāha illallāh, wa ashhadu anna Muḥammadan ʿabduhu wa rasūluh.",
    "meaning": "All honour, worship and good things belong to Allah. Peace, Allah’s mercy and blessings be upon the Prophet. Peace be upon us and Allah’s righteous servants. I testify that none is worthy of worship but Allah, and that Muhammad is His servant and messenger.",
    "pages": "41, 45",
    "note": "A reported wording of the tashahhud. Pronunciation is learned best with a teacher.",
    "link": [
      "Bukhari 6265",
      "https://sunnah.com/bukhari:6265"
    ]
  },
  {
    "id": "salawat",
    "title": "Blessings upon the Prophet",
    "arabic": "اللَّهُمَّ صَلِّ عَلَى مُحَمَّدٍ وَعَلَى آلِ مُحَمَّدٍ كَمَا صَلَّيْتَ عَلَى إِبْرَاهِيمَ وَعَلَى آلِ إِبْرَاهِيمَ إِنَّكَ حَمِيدٌ مَجِيدٌ، اللَّهُمَّ بَارِكْ عَلَى مُحَمَّدٍ وَعَلَى آلِ مُحَمَّدٍ كَمَا بَارَكْتَ عَلَى إِبْرَاهِيمَ وَعَلَى آلِ إِبْرَاهِيمَ إِنَّكَ حَمِيدٌ مَجِيدٌ",
    "translit": "Allāhumma ṣalli ʿalā Muḥammadin wa ʿalā āli Muḥammad, kamā ṣallayta ʿalā Ibrāhīma wa ʿalā āli Ibrāhīm, innaka ḥamīdun majīd. Allāhumma bārik ʿalā Muḥammadin wa ʿalā āli Muḥammad, kamā bārakta ʿalā Ibrāhīma wa ʿalā āli Ibrāhīm, innaka ḥamīdun majīd.",
    "meaning": "O Allah, honour Muhammad and his family as You honoured Ibrahim and his family. You are praiseworthy and glorious. Bless Muhammad and his family as You blessed Ibrahim and his family. You are praiseworthy and glorious.",
    "pages": "41–42, 46",
    "note": "Blessings upon the Prophet ﷺ, recited in the sitting."
  },
  {
    "id": "protection",
    "title": "Protection from four trials",
    "arabic": "اللَّهُمَّ إِنِّي أَعُوذُ بِكَ مِنْ عَذَابِ جَهَنَّمَ وَمِنْ عَذَابِ الْقَبْرِ وَمِنْ فِتْنَةِ الْمَحْيَا وَالْمَمَاتِ وَمِنْ شَرِّ فِتْنَةِ الْمَسِيحِ الدَّجَّالِ",
    "translit": "Allāhumma innī aʿūdhu bika min ʿadhābi jahannam, wa min ʿadhābil-qabr, wa min fitnatil-maḥyā wal-mamāt, wa min sharri fitnatil-masīḥid-dajjāl.",
    "meaning": "O Allah, protect me from Hell’s punishment, the punishment of the grave, the trials of life and death, and the evil trial of the False Messiah.",
    "pages": "46–47",
    "link": [
      "Muslim 588a",
      "https://sunnah.com/muslim:588a"
    ]
  },
  {
    "id": "salam",
    "title": "The closing salutation",
    "arabic": "السَّلَامُ عَلَيْكُمْ وَرَحْمَةُ اللَّهِ",
    "translit": "As-salāmu ʿalaykum wa raḥmatullāh.",
    "meaning": "Peace and Allah’s mercy be upon you.",
    "pages": "47–48"
  },
  {
    "id": "beginner",
    "title": "Remembrance while learning",
    "arabic": "سُبْحَانَ اللَّهِ وَالْحَمْدُ لِلَّهِ وَلَا إِلَهَ إِلَّا اللَّهُ وَاللَّهُ أَكْبَرُ وَلَا حَوْلَ وَلَا قُوَّةَ إِلَّا بِاللَّهِ",
    "translit": "Subḥānallāh, wal-ḥamdu lillāh, wa lā ilāha illallāh, wallāhu akbar, wa lā ḥawla wa lā quwwata illā billāh.",
    "meaning": "Allah is free of imperfection; praise belongs to Him. None is worthy of worship but Allah. Allah is greater than all. There is no power or strength except through Allah.",
    "pages": "28–29",
    "note": "A temporary concession for genuine inability while learning Al-Fātiḥah. Learning continues with a teacher’s help."
  },
  {
    "id": "after-forgive",
    "title": "After prayer · seeking forgiveness",
    "arabic": "أَسْتَغْفِرُ اللَّهَ",
    "translit": "Astaghfirullāh.",
    "meaning": "I seek Allah’s forgiveness.",
    "note": "Said three times after completing the prayer.",
    "link": [
      "Sahih Muslim 591",
      "https://sunnah.com/muslim:591"
    ]
  },
  {
    "id": "after-peace",
    "title": "After prayer · the words of peace",
    "arabic": "اللَّهُمَّ أَنْتَ السَّلاَمُ وَمِنْكَ السَّلاَمُ تَبَارَكْتَ ذَا الْجَلاَلِ وَالإِكْرَامِ",
    "translit": "Allāhumma antas-salāmu wa minkas-salām, tabārakta dhal-jalāli wal-ikrām.",
    "meaning": "O Allah, You are Peace, and from You comes peace. Blessed are You, Possessor of majesty and honour.",
    "note": "Recited after seeking forgiveness three times. English is a study rendering of the meaning.",
    "link": [
      "Sahih Muslim 591",
      "https://sunnah.com/muslim:591"
    ]
  },
  {
    "id": "after-praise",
    "title": "After prayer · glorification & praise",
    "arabic": "سُبْحَانَ اللَّهِ · الْحَمْدُ لِلَّهِ · اللَّهُ أَكْبَرُ",
    "translit": "Subḥānallāh. Alḥamdu lillāh. Allāhu akbar.",
    "meaning": "Allah is free of imperfection. Praise belongs to Allah. Allah is greater than all.",
    "note": "Each phrase is said thirty-three times in the practice reported in Sahih Muslim 597a. The declaration of oneness completes a hundred.",
    "link": [
      "Sahih Muslim 597a",
      "https://sunnah.com/muslim:597a"
    ]
  },
  {
    "id": "after-tahlil",
    "title": "After prayer · the declaration of oneness",
    "arabic": "لاَ إِلَهَ إِلاَّ اللَّهُ وَحْدَهُ لاَ شَرِيكَ لَهُ لَهُ الْمُلْكُ وَلَهُ الْحَمْدُ وَهُوَ عَلَى كُلِّ شَىْءٍ قَدِيرٌ",
    "translit": "Lā ilāha illallāhu waḥdahu lā sharīka lah, lahul-mulku wa lahul-ḥamd, wa huwa ʿalā kulli shayʾin qadīr.",
    "meaning": "None has the right to be worshipped but Allah alone, without partner. To Him belongs dominion and praise, and He has power over all things.",
    "note": "Said once to complete a hundred after the three phrases said thirty-three times each. English is a study rendering of the meaning.",
    "link": [
      "Sahih Muslim 597a",
      "https://sunnah.com/muslim:597a"
    ]
  }
];
