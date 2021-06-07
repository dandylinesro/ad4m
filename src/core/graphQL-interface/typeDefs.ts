import { gql } from 'apollo-server'


const typeDefs = gql`
scalar Date

type Agent {
    did: String
    name: String
    email: String
}

type AgentService {
    agent: Agent
    isInitialized: Boolean
    isUnlocked: Boolean
    did: String
    didDocument: String
    error: String
}

input InitializeAgent {
    did: String
    didDocument: String
    keystore: String
    passphrase: String
}

type Icon {
    code: String
}

type Expression {
    url: String

    author: Agent
    timestamp: String
    data: String

    icon: Icon
    language: Language

    proof: ExpressionProof
}

type ExpressionProof {
    signature: String
    key: String
    valid: Boolean
    invalid: Boolean
}

type Link {
    source: String
    predicate: String
    target: String
}

type LinkExpression {
    author: Agent
    timestamp: String
    data: Link
}

input LinkQuery {
    source: String
    predicate: String
    target: String
    fromDate: Date
    untilDate: Date
}

type Language {
    name: String
    address: String
    constructorIcon: Icon
    iconFor: Icon
    settings: String
    settingsIcon: Icon
}

type Perspective {
    uuid: String
    name: String
    sharedPerspective: SharedPerspective
    sharedURL: String
    links(query: LinkQuery): [LinkExpression]
}

type SharedPerspective {
    name: String
    description: String
    type: String
    linkLanguages: [LanguageRef]
    allowedExpressionLanguages: [String]
    requiredExpressionLanguages: [String]
    sharedExpressionLanguages: [String]
}

type LanguageRef {
    address: String
    name: String
}

input LanguageRefInput {
    address: String
    name: String
}

type Signal {
    language: String
    signal: String
}

input AddLinkInput {
    perspectiveUUID: String
    link: String
}

input UpdateLinkInput {
    perspectiveUUID: String
    oldLink: String
    newLink: String
}

input RemoveLinkInput {
    perspectiveUUID: String
    link: String
}

input CreateExpressionInput {
    languageAddress: String
    content: String
}

input SetLanguageSettingsInput {
    languageAddress: String
    settings: String
}

input AddPerspectiveInput {
    name: String
}

input UpdatePerspectiveInput {
    uuid: String
    name: String
    linksSharingLanguage: String
}

input PublishPerspectiveInput {
    uuid: String
    name: String
    description: String
    type: String
    uid: String
    requiredExpressionLanguages: [String]
    allowedExpressionLanguages: [String]
}

input CreateHcExpressionLanguageInput {
    languagePath: String 
    dnaNick: String 
    uid: String
}

input UpdateAgentProfileInput {
    name: String
    email: String
}

type Query {
    hello: String
    agent: AgentService
    links(perspectiveUUID: String, query: LinkQuery): [LinkExpression]
    expression(url: String): Expression
    expressionRaw(url: String): String
    language(address: String): Language
    languages(filter: String): [Language]
    perspectives: [Perspective]
    perspective(uuid: String): Perspective
    pubKeyForLanguage(lang: String): String
}

type Mutation {
    initializeAgent(input: InitializeAgent): AgentService
    lockAgent(passphrase: String): AgentService
    unlockAgent(passphrase: String): AgentService
    updateAgentProfile(input: UpdateAgentProfileInput): AgentService
    addPerspective(input: AddPerspectiveInput): Perspective
    updatePerspective(input: UpdatePerspectiveInput): Perspective
    removePerspective(uuid: String): Boolean
    publishPerspective(input: PublishPerspectiveInput): SharedPerspective
    installSharedPerspective(url: String): Perspective
    addLink(input: AddLinkInput): LinkExpression
    updateLink(input: UpdateLinkInput): LinkExpression
    removeLink(input: RemoveLinkInput): Boolean
    createExpression(input: CreateExpressionInput): String
    setLanguageSettings(input: SetLanguageSettingsInput): Boolean
    openLinkExtern(url: String): Boolean
    quit: Boolean
    createUniqueHolochainExpressionLanguageFromTemplate(input: CreateHcExpressionLanguageInput): LanguageRef
}

type Subscription {
    agentUpdated: Agent
    perspectiveAdded: Perspective
    perspectiveUpdated: Perspective
    perspectiveRemoved: String
    linkAdded(perspectiveUUID: String): LinkExpression
    linkRemoved(perspectiveUUID: String): LinkExpression
    signal: Signal
}
`

export default typeDefs