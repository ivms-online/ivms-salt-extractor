##
# This file is part of the IVMS Online.
#
# @copyright 2023 © by Rafał Wrzeszcz - Wrzasq.pl.
##

Feature: Generate JWT token

    Scenario: Licences JWT generation
        Given There is an inventory "test0" of type "jwt_key" for vessel "00000000-0000-0000-0000-000000000000" of customer "00000000-0000-0000-0000-000000000001" with serial number "qwerta"
        And There is an inventory "test2" of type "jwt_key" for vessel "00000000-0000-0000-0000-000000000000" of customer "00000000-0000-0000-0000-000000000001" with serial number "qwertp"
        And There is a license "key0" for vessel "00000000-0000-0000-0000-000000000000" of customer "00000000-0000-0000-0000-000000000001" with count 2 and expiration date "2011-01-30T14:58:00+01:00"
        And There is a license "weather" for vessel "00000000-0000-0000-0000-000000000000" of customer "00000000-0000-0000-0000-000000000001" with count 4 and expiration date "2017-11-11T16:00:00+02:00"
        And There is a license "other" for vessel "00000000-0000-0000-0000-000000000002" of customer "00000000-0000-0000-0000-000000000001" with count 2 and expiration date "2011-01-30T14:58:00+01:00"
        When I request JWT token for vessel "00000000-0000-0000-0000-000000000000" of customer "00000000-0000-0000-0000-000000000001" with "integration-test" issuer for "ivms-host" audience with "test0" specified as verification key
        Then I can verify JWT claims with key "qwerta"
        And I have JWT token with "integration-test" issuer claim
        And I have JWT token for "ivms-host" audience claim
        And I have JWT token for "00000000-0000-0000-0000-000000000001:00000000-0000-0000-0000-000000000000" sub user claim
        And I can find license "key0" with count 2 and expiration date "2011-01-30T14:58:00+01:00" in JWT claims
        And I can find license "weather" with count 4 and expiration date "2017-11-11T16:00:00+02:00" in JWT claims
        And I can not find license "other" in JWT claims
