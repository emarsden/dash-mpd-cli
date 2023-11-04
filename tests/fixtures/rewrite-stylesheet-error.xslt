<?xml version="1.0" encoding="utf-8"?>
<xsl:stylesheet version="1.0"
                xmlns:xsl="http://www.w3.org/1999/XSL/Transform"
                xmlns:mpd="urn:mpeg:dash:schema:mpd:2011">
  <xsl:output method="xml" indent="yes"/>

  <!--
      This stylesheet is deliberately erroneous, for use in the test test_xslt_stylesheet_error in
      test/xslt.rs.
  -->
  <xsl:template match="//fooble()[local-name()='FOOBLE' and starts-with(@mimeType,'audio/')]" />
</xsl:stylesheet>
