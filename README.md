# Overview

This project is a restful backend for retrograde, an online agile retrospective tool.

# MVP:
  - make a board
  - view board as owner
  - add/edit/delete columns
  - public link to view board as participant
  - add/edit/delete cards
  - button to stop new cards
  - button to open voting
  - button to close voting
  - vote on a card
  - remove a vote
  - sort by vote count
  - single list result view sorted by vote count.

# Notes:
  - no accounts
  - no card comments
  - boards expire after some time of no use
  - all users get a cookie token
  - max votes are based on the cookie token
  - board admin is based on the creators cookie token
  - google cloud run
  - google (p)sql

# Routes:
  - post    /boards [name, max_votes, voting_open, cards_open] -> id
  - get     /boards [token] -> list of boards
  - get     /boards/:id -> owner, name, max_votes, voting_open, cards_open
  - patch   /boards/:id [name, max_votes, voting_open, cards_open]
  - delete  /boards/:id

  - post    /boards/:id/participant -> token	// maybe just 'get' can auto add participant

  - post    /boards/:id/columns [name] -> id
  - get     /boards/:id/columns -> list of columns
  - get     /boards/:id/columns/:id -> name
  - patch   /boards/:id/columns/:id [name]
  - delete  /boards/:id/columns/:id

  - post    /boards/:id[/columns/:id]/cards [name, description] -> id
  - get     /boards/:id[/columns/:id]/cards -> list of cards
  - get     /boards/:id[/columns/:id]/cards/:id -> name, description, votes, user_votes
  - patch   /boards/:id[/columns/:id]/cards/:id [name, description]
  - delete  /boards/:id[/columns/:id]/cards/:id

  - post    /boards/:id[/columns/:id]/cards/:id/vote [token]
  - post    /boards/:id[/columns/:id]/cards/:id/unvote [token]

# ORM:
  - board:
    - id
    - name
    - max_votes
    - voting_open
    - cards_open

  - participant:
    - id
    - board id
    - owner (bool)
    - token

  - rank:
    - id
    - board id
    - name

  - card:
    - id
    - rank id
    - name
    - description

  - vote:
    - id
    - card_id
    - user_token

# Future ideas:
  - reactions on cards (emoji)
  - save completed board as image/pdf
  - participant and voting stats per board