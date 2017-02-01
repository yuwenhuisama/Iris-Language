#include "IrisInterpreter\IrisNativeClasses\IrisIrregularTag.h"



IrisIrregularTag::IrisIrregularTag()
{
}

void IrisIrregularTag::Initialize(size_t nLineNumber, const string& strFileName, const string& strMessage)
{
	m_nLineNumber = nLineNumber;
	m_strFileName = strFileName;
	m_strMessage = strMessage;
}


IrisIrregularTag::~IrisIrregularTag()
{
}
