'use strict';
const copy = {
  en: {
    pageTitle:'Ignis Engine — Ignite the spark. Create worlds.', meta:'Ignis is a 3D engine in Rust, built around a shared creative loop for humans and AI agents. From the spark of an idea to a playable world.',
    skip:'Skip to content',navigation:'Main navigation',navVision:'The vision',navLab:'Inside the engine',navRoad:'The road ahead',navExplore:'Explore Ignis',
    heroEyebrow:'A NEW SPARK IN GAME CREATION',heroOne:'Ignite the spark.',heroTwo:'Create worlds.',heroDescription:'A 3D engine built for the way you imagine. Human creativity. Intelligent tools. One shared world.',heroCta:'Discover the vision',scroll:'A world begins below',conceptArt:'CONCEPTUAL BRAND ARTWORK',heroAlt:'A glowing ember portal in an obsidian landscape. Conceptual brand artwork.',
    development:'IN ACTIVE DEVELOPMENT',rust:'RUST AT THE CORE',mcp:'MCP FROM THE START',human:'HUMANS IN CONTROL',
    visionEyebrow:'THE IDEA BEHIND IGNIS',visionTitle:'Great worlds<br>start with<br><em>a little spark.</em>',visionLead:"That idea you can't stop thinking about? Give it somewhere to exist.",visionBody:'Ignis is a 3D engine in Rust designed for humans and AI agents to work on the same project. Describe, build, play, observe, refine. Keep your creative intent at the center of every iteration.',visionBodyTwo:"We're starting with the foundations: a real simulation, inspectable decisions and changes you can undo. A small, deliberate beginning for a much bigger vision.",visionLink:'Meet the creative loop',
    principleOne:'Your world. Your intent.',principleOneBody:'The creative direction stays with you. Tools help you make it tangible.',principleTwo:'One engine. Shared control.',principleTwoBody:'Native controls and MCP commands reach the same simulation.',principleThree:'Room to experiment.',principleThreeBody:'Inspect what happened, undo an edit and try another direction.',
    loopEyebrow:'THE CREATIVE LOOP',loopSide:'IDEA → ITERATION → POSSIBILITY',loopTitle:'Make something.<br><em>Then make it yours.</em>',loopIntro:"Creation is a conversation with what you're building. Ignis is being built around that exchange.",orbitalCaption:'IGNIS / CREATIVE SYSTEM',loopLabel:'Creative loop',stage0:'Describe',stage1:'Build',stage2:'Play',stage3:'Observe',stage4:'Refine',panelLabel:'FROM INTENT TO ITERATION',commandNote:'COMMAND FLOW',
    labEyebrow:'SMALL BEGINNING. REAL FOUNDATIONS.',labSide:'THE CURRENT LABORATORY',labTitle:'A spark you<br>can <em>already see.</em>',labLead:'The first chapter is running.',labBody:'A native 3D laboratory with procedural rooms, physics and NPCs that see, remember, search and investigate sound cues. Every step can be inspected through MCP.',labWindow:'PERCEPTION LAB',labPlatform:'macOS · Metal',realCapture:'ACTUAL ENGINE CAPTURE',labAlt:'Actual Ignis laboratory capture: a purple NPC, orange player and green barrier with a vision guide.',labCaption:'Simple geometry. Inspectable behavior. The foundations come first.',labCaptionTwo:'Local laboratory · Not a public release',
    capOne:'Worlds with a starting point.',capOneBody:'Generate rooms from parameters and seeds. Edit versioned scenes with stable IDs, validation and undo/redo.',capTagOne:'PROCEDURAL SCENES',capTwo:'Behavior you can understand.',capTwoBody:'NPCs act on sight, memory and optional sound cues. Inspect their known information, search destinations and reasons for acting.',capTagTwo:'PERCEPTION & MEMORY',capThree:'A closer look at every step.',capThreeBody:'Advance the simulation, inspect events and capture the scene. Replay inputs on the same build to investigate what happened.',capTagThree:'INSPECT & REPLAY',labLimit:'This is an early laboratory, not the full engine. macOS is the first validated platform. Advanced rendering, animation, asset import and multiplayer are still ahead.',
    roadEyebrow:'A BIGGER HORIZON',roadSide:'BUILT ONE FOUNDATION AT A TIME',roadTitle:'The spark<br>is just<br><em>the beginning.</em>',roadIntro:'The ambition is a modular engine for building worlds with intelligent tools, inspectable systems and creative freedom.',roadOne:'Lay the foundations',roadNow:'NOW',roadOneBody:"The current laboratory brings together procedural rooms, physics, NPC perception, reversible editing and MCP control. We're proving the creative loop with real, inspectable systems.",roadTwo:'Expand the creative toolkit',roadPlanned:'PLANNED',roadTwoBody:'Richer scenes, asset workflows, materials, animation and a visual editor with the same capabilities exposed to agents. These are roadmap goals, not available features.',roadThree:'Grow into living worlds',roadThreeBody:'Composable gameplay, characters, quests, world streaming and networked play. Each system needs its own contracts, demonstrations and validation. No release dates are committed.',
    closingEyebrow:'FROM THE SPARK OF AN IDEA TO A PLAYABLE WORLD',closingTitle:'Every world<br>starts <em>somewhere.</em>',closingCta:'Start with a spark',closingNote:'Ignis Engine. An ambitious beginning, built with intention.',footerLine:'The spark is yours.\nThe possibilities are ahead.',brandLink:'Explore the identity',footerStatus:'In development · Public release not yet available',backTop:'Back to top',
    brandPageTitle:'Ignis — A spark with an identity',brandEyebrow:'THE IGNIS IDENTITY',brandTitle:'A spark.<br><em>An identity.</em>',brandIntro:'A flame on the outside. A doorway within. The beginning of a world, in one mark.',symbolLabel:'01 / The symbol',lockupLabel:'02 / The full signature',download:'Download PNG ↓',brandBack:'Back to Ignis',brandMeaning:'Energy. An opening. Possibility.',brandMeaningBody:'The ascending flame represents creative energy. The opening at its center turns that spark into a portal: an invitation to build a world. The signature pairs the symbol with architectural lettering, designed for a dark background.',brandLineTitle:'One thought. Two languages.',brandLine:'Ignite the spark. Create worlds.',brandLinePt:'Acenda a centelha. Crie mundos.',brandNote:'First visual proposal · Ready for creative review',
    stages:[
      ['Start with an intention.','Set the direction. An agent can translate your intent into explicit scene commands, while the engine validates each change.','generate → inspect → upsert'],
      ['Give the idea a shape.','Generate a room from a seed. Add and configure entities with stable IDs. Scene edits are validated and can be undone.','generate → upsert → save_scene'],
      ['Put it into motion.','Run the native laboratory or advance it in fixed steps. Keyboard controls and MCP commands share the same simulation.','set_input → step → inspect'],
      ['Understand what happened.','Capture the scene and inspect NPC perception, remembered positions and navigation decisions. Separate what is seen from what is assumed.','capture → inspect_npcs → inspect_navigation'],
      ['Try another direction.','Undo an edit, change the scene and run again. Replay inputs on the same build to investigate behavior from a known starting point.','undo → upsert → step']
    ]
  },
  pt: {
    pageTitle:'Ignis Engine — Acenda a centelha. Crie mundos.',meta:'Ignis é uma engine 3D em Rust, pensada para humanos e agentes de IA criarem juntos. Da centelha da ideia a um mundo jogável.',
    skip:'Pular para o conteúdo',navigation:'Navegação principal',navVision:'A visão',navLab:'Por dentro da engine',navRoad:'O próximo horizonte',navExplore:'Explore a Ignis',
    heroEyebrow:'UMA NOVA CENTELHA NA CRIAÇÃO DE JOGOS',heroOne:'Acenda a centelha.',heroTwo:'Crie mundos.',heroDescription:'Uma engine 3D para dar forma à sua imaginação. Criatividade humana. Ferramentas inteligentes. Um mundo compartilhado.',heroCta:'Conheça a visão',scroll:'Um mundo começa aqui',conceptArt:'ARTE CONCEITUAL DA MARCA',heroAlt:'Um portal de brasa ilumina uma paisagem de obsidiana. Arte conceitual da marca.',
    development:'EM DESENVOLVIMENTO ATIVO',rust:'RUST NO NÚCLEO',mcp:'MCP DESDE O INÍCIO',human:'HUMANOS NO CONTROLE',
    visionEyebrow:'A IDEIA POR TRÁS DA IGNIS',visionTitle:'Grandes mundos<br>começam com<br><em>uma centelha.</em>',visionLead:'Aquela ideia que não sai da sua cabeça? Dê a ela um lugar para existir.',visionBody:'Ignis é uma engine 3D em Rust pensada para humanos e agentes de IA trabalharem no mesmo projeto. Descrever, construir, jogar, observar e refinar. Sua intenção criativa no centro de cada iteração.',visionBodyTwo:'Começamos pelos fundamentos: uma simulação real, decisões que podem ser inspecionadas e alterações que você pode desfazer. Um começo pequeno e intencional para uma visão muito maior.',visionLink:'Conheça o ciclo criativo',
    principleOne:'Seu mundo. Sua intenção.',principleOneBody:'A direção criativa continua com você. As ferramentas ajudam a torná-la concreta.',principleTwo:'Uma engine. Controle compartilhado.',principleTwoBody:'Controles nativos e comandos MCP acessam a mesma simulação.',principleThree:'Espaço para experimentar.',principleThreeBody:'Inspecione o que aconteceu, desfaça uma edição e explore outra direção.',
    loopEyebrow:'O CICLO CRIATIVO',loopSide:'IDEIA → ITERAÇÃO → POSSIBILIDADE',loopTitle:'Crie algo.<br><em>Depois, dê a sua forma.</em>',loopIntro:'Criar é conversar com aquilo que você está construindo. A Ignis está nascendo em torno dessa troca.',orbitalCaption:'IGNIS / SISTEMA CRIATIVO',loopLabel:'Ciclo criativo',stage0:'Descrever',stage1:'Construir',stage2:'Jogar',stage3:'Observar',stage4:'Refinar',panelLabel:'DA INTENÇÃO À ITERAÇÃO',commandNote:'FLUXO DE COMANDOS',
    labEyebrow:'PEQUENO COMEÇO. FUNDAMENTOS REAIS.',labSide:'O LABORATÓRIO ATUAL',labTitle:'Uma centelha<br>que <em>já dá para ver.</em>',labLead:'O primeiro capítulo já está rodando.',labBody:'Um laboratório 3D nativo com salas procedurais, física e NPCs que enxergam, lembram, buscam e investigam pistas sonoras. Cada passo pode ser inspecionado pelo MCP.',labWindow:'LABORATÓRIO DE PERCEPÇÃO',labPlatform:'macOS · Metal',realCapture:'CAPTURA REAL DA ENGINE',labAlt:'Captura real do laboratório Ignis: NPC roxo, jogador laranja e barreira verde com guia de visão.',labCaption:'Geometria simples. Comportamento observável. Os fundamentos vêm primeiro.',labCaptionTwo:'Laboratório local · Ainda sem versão pública',
    capOne:'Mundos com um ponto de partida.',capOneBody:'Gere salas a partir de parâmetros e sementes. Edite cenas versionadas com IDs estáveis, validação e histórico para desfazer e refazer.',capTagOne:'CENAS PROCEDURAIS',capTwo:'Comportamentos que fazem sentido.',capTwoBody:'NPCs agem com base na visão, memória e pistas sonoras opcionais. Inspecione o que sabem, onde procuram e por que estão agindo.',capTagTwo:'PERCEPÇÃO E MEMÓRIA',capThree:'Cada passo sob um novo olhar.',capThreeBody:'Avance a simulação, inspecione eventos e capture a cena. Reproduza entradas no mesmo build para investigar o que aconteceu.',capTagThree:'INSPEÇÃO E REPLAY',labLimit:'Este é um laboratório inicial, ainda não a engine completa. macOS é a primeira plataforma validada. Renderização avançada, animação, importação de assets e multiplayer estão no horizonte.',
    roadEyebrow:'UM HORIZONTE MAIOR',roadSide:'UM FUNDAMENTO DE CADA VEZ',roadTitle:'A centelha<br>é só<br><em>o começo.</em>',roadIntro:'A ambição é uma engine modular para construir mundos com ferramentas inteligentes, sistemas observáveis e liberdade criativa.',roadOne:'Firmar os fundamentos',roadNow:'AGORA',roadOneBody:'O laboratório atual reúne salas procedurais, física, percepção de NPCs, edição reversível e controle por MCP. Estamos demonstrando o ciclo criativo com sistemas reais e observáveis.',roadTwo:'Ampliar as ferramentas de criação',roadPlanned:'PLANEJADO',roadTwoBody:'Cenas mais ricas, fluxos de assets, materiais, animação e um editor visual com as mesmas capacidades expostas aos agentes. São objetivos do roadmap, ainda não funcionalidades disponíveis.',roadThree:'Dar vida a mundos maiores',roadThreeBody:'Gameplay por composição, personagens, quests, carregamento de mundos e partidas em rede. Cada sistema precisa de contratos, demonstrações e validação próprios. Ainda não há datas de lançamento.',
    closingEyebrow:'DA CENTELHA DA IDEIA A UM MUNDO JOGÁVEL',closingTitle:'Todo mundo<br>tem <em>um começo.</em>',closingCta:'Comece com uma centelha',closingNote:'Ignis Engine. Um começo ambicioso, construído com intenção.',footerLine:'A centelha é sua.\nAs possibilidades estão à frente.',brandLink:'Explore a identidade',footerStatus:'Em desenvolvimento · Versão pública ainda indisponível',backTop:'Voltar ao início',
    brandPageTitle:'Ignis — Uma centelha com identidade',brandEyebrow:'A IDENTIDADE IGNIS',brandTitle:'Uma centelha.<br><em>Uma identidade.</em>',brandIntro:'Por fora, uma chama. Por dentro, uma passagem. O começo de um mundo em um único símbolo.',symbolLabel:'01 / O símbolo',lockupLabel:'02 / A assinatura completa',download:'Baixar PNG ↓',brandBack:'Voltar para a Ignis',brandMeaning:'Energia. Uma passagem. Possibilidade.',brandMeaningBody:'A chama ascendente representa a energia criativa. A abertura no centro transforma essa centelha em um portal: um convite para construir um mundo. A assinatura combina o símbolo com letras de desenho arquitetônico, pensadas para fundo escuro.',brandLineTitle:'Uma ideia. Duas línguas.',brandLine:'Acenda a centelha. Crie mundos.',brandLinePt:'Ignite the spark. Create worlds.',brandNote:'Primeira proposta visual · Pronta para revisão criativa',
    stages:[
      ['Comece com uma intenção.','Defina a direção. Um agente pode traduzir sua intenção em comandos explícitos de cena, enquanto a engine valida cada alteração.','generate → inspect → upsert'],
      ['Dê forma à ideia.','Gere uma sala a partir de uma semente. Adicione e configure entidades com IDs estáveis. As edições são validadas e podem ser desfeitas.','generate → upsert → save_scene'],
      ['Coloque tudo em movimento.','Execute o laboratório nativo ou avance em passos fixos. Controles de teclado e comandos MCP compartilham a mesma simulação.','set_input → step → inspect'],
      ['Entenda o que aconteceu.','Capture a cena e inspecione percepção, posições lembradas e decisões de navegação dos NPCs. Separe o que foi visto do que é uma hipótese.','capture → inspect_npcs → inspect_navigation'],
      ['Experimente outra direção.','Desfaça uma edição, altere a cena e execute de novo. Reproduza entradas no mesmo build para investigar o comportamento a partir de um estado conhecido.','undo → upsert → step']
    ]
  }
};
const textNodes = [...document.querySelectorAll('[data-i18n]')];
const richNodes = [...document.querySelectorAll('[data-i18n-html]')];
const ariaNodes = [...document.querySelectorAll('[data-aria]')];
const altNodes = [...document.querySelectorAll('[data-alt]')];
const languageButtons = [...document.querySelectorAll('[data-lang]')];
const stageButtons = [...document.querySelectorAll('[data-stage]')];
let language = 'en';
let currentStage = 0;
function setStage(next, focus = false) {
  if (!stageButtons.length) return;
  currentStage = (next + stageButtons.length) % stageButtons.length;
  stageButtons.forEach((button,index) => {
    const selected = index === currentStage;
    button.classList.toggle('active',selected);
    button.setAttribute('aria-selected',String(selected));
    button.tabIndex = selected ? 0 : -1;
  });
  const stage = copy[language].stages[currentStage];
  document.getElementById('stage-title').textContent = stage[0];
  document.getElementById('stage-description').textContent = stage[1];
  document.getElementById('stage-code').textContent = stage[2];
  document.querySelector('.panel-count').textContent = `0${currentStage+1} / 05`;
  const panel = document.getElementById('stage-panel');
  panel.setAttribute('aria-labelledby',`stage-${currentStage}`);
  panel.dataset.current = currentStage;
  if (focus) stageButtons[currentStage].focus();
}
function setLanguage(next, persist = true) {
  language = Object.hasOwn(copy,next) ? next : 'en';
  const dictionary = copy[language];
  document.documentElement.lang = language === 'pt' ? 'pt-BR' : 'en';
  textNodes.forEach(node => { node.textContent = dictionary[node.dataset.i18n]; });
  // Rich copy is a fixed, local dictionary, never visitor or external content.
  richNodes.forEach(node => { node.innerHTML = dictionary[node.dataset.i18nHtml]; });
  ariaNodes.forEach(node => { node.setAttribute('aria-label',dictionary[node.dataset.aria]); });
  altNodes.forEach(node => { node.alt = dictionary[node.dataset.alt]; });
  languageButtons.forEach(button => button.setAttribute('aria-pressed',String(button.dataset.lang === language)));
  const identity = document.body.classList.contains('brand-page');
  document.title = identity ? dictionary.brandPageTitle : dictionary.pageTitle;
  const meta = document.querySelector('meta[name="description"]');
  if (meta) meta.content = identity ? dictionary.brandIntro : dictionary.meta;
  document.querySelectorAll('[data-locale-link]').forEach(link => {
    link.href = `${link.dataset.localeLink}?lang=${language}`;
  });
  setStage(currentStage);
  if (persist) {
    try { localStorage.setItem('ignis-language',language); } catch {}
    const url = new URL(location.href);
    url.searchParams.set('lang',language);
    history.replaceState(null,'',url);
  }
}
document.querySelectorAll('a[href="brand.html"],a[href="index.html"]').forEach(link => {
  link.dataset.localeLink = link.getAttribute('href');
});
languageButtons.forEach(button => button.addEventListener('click',() => setLanguage(button.dataset.lang)));
stageButtons.forEach((button,index) => {
  button.addEventListener('click',() => setStage(index));
  button.addEventListener('keydown',event => {
    let next;
    if (['ArrowDown','ArrowRight'].includes(event.key)) next = currentStage+1;
    if (['ArrowUp','ArrowLeft'].includes(event.key)) next = currentStage-1;
    if (event.key === 'Home') next = 0;
    if (event.key === 'End') next = stageButtons.length-1;
    if (next !== undefined) { event.preventDefault(); setStage(next,true); }
  });
});
function initialLanguage() {
  const query = new URL(location.href).searchParams.get('lang');
  if (query && Object.hasOwn(copy,query)) return query;
  try {
    const remembered = localStorage.getItem('ignis-language');
    if (remembered && Object.hasOwn(copy,remembered)) return remembered;
  } catch {}
  return navigator.language.toLowerCase().startsWith('pt') ? 'pt' : 'en';
}
setLanguage(initialLanguage(),false);
const media = window.matchMedia('(max-width:760px)');
function setTabOrientation() { document.querySelector('[role="tablist"]')?.setAttribute('aria-orientation',media.matches ? 'horizontal' : 'vertical'); }
setTabOrientation();
media.addEventListener('change',setTabOrientation);
if ('IntersectionObserver' in window && !matchMedia('(prefers-reduced-motion:reduce)').matches) {
  const observer = new IntersectionObserver(entries => entries.forEach(entry => {
    if (entry.isIntersecting) { entry.target.classList.add('visible'); observer.unobserve(entry.target); }
  }),{threshold:0.06});
  document.querySelectorAll('.vision-grid,.principles,.section-heading,.loop-workspace,.lab-heading,.lab-figure,.capability-list,.road-grid,.closing-inner').forEach(node => {
    node.classList.add('reveal');
    observer.observe(node);
  });
}
