#include "IrisComponents/IrisExpressions/IrisNativeObjectExpression.h"
#include "IrisCompiler.h"
#include "IrisInstructorMaker.h"


bool IrisNativeObjectExpression::Generate()
{
	auto pCompiler = IrisCompiler::CurrentCompiler();
	auto pMaker = IrisInstructorMaker::CurrentInstructor();
	pCompiler->SetLineNumber(m_nLineNumber);

	switch (m_eType)
	{
	case IrisNativeObjectType::String:
		pMaker->load(IrisAMType::ImmediateString, pCompiler->GetStringIndex(m_strString, pCompiler->GetCurrentFileIndex()));
		break;
	case IrisNativeObjectType::UniqueString:
		pMaker->load(IrisAMType::ImmediateUniqueString, pCompiler->GetUniqueStringIndex(m_strString, pCompiler->GetCurrentFileIndex()));
		break;
	case IrisNativeObjectType::Integer:
		pMaker->load(IrisAMType::ImmediateInteger, pCompiler->GetIntegerIndex(m_nInteger, pCompiler->GetCurrentFileIndex()));
		break;
	case IrisNativeObjectType::Float:
		pMaker->load(IrisAMType::ImmediateFloat, pCompiler->GetFloatIndex(m_dFloat, pCompiler->GetCurrentFileIndex()));
		break;
	default:
		break;
	}

	return true;
}

IrisNativeObjectExpression::IrisNativeObjectExpression(IrisNativeObjectType eType, const string& strString) : m_eType(eType), m_strString(strString)
{
}

IrisNativeObjectExpression::IrisNativeObjectExpression(int nInteger) : m_eType(IrisNativeObjectType::Integer), m_nInteger(nInteger)
{
}

IrisNativeObjectExpression::IrisNativeObjectExpression(double dFloat) : m_eType(IrisNativeObjectType::Float), m_dFloat(dFloat)
{
}

IrisNativeObjectExpression::~IrisNativeObjectExpression()
{
}
