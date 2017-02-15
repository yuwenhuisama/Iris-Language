#ifndef _H_IRISSUPERSTATEMENT_
#define _H_IRISSUPERSTATEMENT_
#include "IrisStatement.h"
#include "IrisUnil/IrisList.h"

class IrisExpression;
class IrisSuperStatement :
	public IrisStatement
{
protected:
	IrisList<IrisExpression*>* m_pParameters = nullptr;

public:

	virtual bool Generate();

	IrisSuperStatement(IrisList<IrisExpression*>* pParameters);
	virtual ~IrisSuperStatement();

	virtual bool Validate() override;
};

#endif